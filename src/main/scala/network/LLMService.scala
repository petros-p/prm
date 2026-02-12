package network

import java.net.URI
import java.net.http.{HttpClient, HttpRequest, HttpResponse}
import java.time.Duration
import ujson._

/**
 * Parsed interaction fields extracted by the LLM from natural language input.
 */
case class ParsedInteraction(
  personNames: List[String],
  medium: String,
  location: String,
  theirLocation: Option[String],
  topics: List[String],
  note: Option[String],
  date: Option[String]
)

/**
 * Client for the OpenRouter API that extracts structured interaction data
 * from natural language descriptions.
 */
object LLMService {

  private val apiUrl = "https://openrouter.ai/api/v1/chat/completions"
  private val model = "openai/gpt-4o-mini"

  /**
   * Reads the OpenRouter API key from the OPENROUTER_API_KEY environment variable.
   * Returns Left with an error message if not set.
   */
  def getApiKey: Either[String, String] = {
    val key = System.getenv("OPENROUTER_API_KEY")
    if (key == null || key.trim.isEmpty)
      Left("OPENROUTER_API_KEY environment variable is not set. Get a free key at https://openrouter.ai/keys")
    else
      Right(key.trim)
  }

  /**
   * Calls the LLM to extract structured interaction fields from a natural language description.
   *
   * @param input       The natural language description (e.g., "Had coffee with John at Starbucks")
   * @param knownNames  List of known contact names for accurate matching
   * @return Either an error message or the parsed interaction fields
   */
  def parseInteraction(input: String, knownNames: List[String], apiKey: String): Either[String, ParsedInteraction] = {
    val namesStr = knownNames.mkString(", ")
    val today = java.time.LocalDate.now().toString
    val systemPrompt =
      s"""You extract interaction metadata from natural language descriptions.
         |Today's date is $today.
         |Known contacts: [$namesStr]
         |Respond with JSON only, no other text.
         |JSON schema: { "personNames": ["..."], "medium": "InPerson|Text|PhoneCall|VideoCall|SocialMedia", "location": "...", "theirLocation": null, "topics": ["..."], "note": null, "date": null }
         |Rules:
         |- personNames is an array of ALL people mentioned in the input. If multiple people are mentioned, include all of them. Use names exactly as written. Match to known contacts when possible. NEVER substitute different names.
         |- If the medium is not mentioned, default to "InPerson"
         |- location: use the most specific location from the input (e.g. "Charlton, MA" not just "home"). Include the full place name, city, or address as given.
         |- theirLocation is only for remote interactions where their location differs; set to null for in-person
         |- topics: ONLY include activities or subjects explicitly mentioned in the input. Do NOT infer or add topics that weren't stated. Be aware of slang (e.g. "gas" means great/amazing, not cooking).
         |- note is for any additional context not captured in other fields; set to null if none
         |- date: ONLY set to a "YYYY-MM-DD" string if the user explicitly mentions a specific date (e.g. "yesterday", "last Friday", "on March 5th"). Otherwise MUST be null. null means today.""".stripMargin

    val requestBody = ujson.Obj(
      "model" -> model,
      "response_format" -> ujson.Obj("type" -> "json_object"),
      "messages" -> ujson.Arr(
        ujson.Obj("role" -> "system", "content" -> systemPrompt),
        ujson.Obj("role" -> "user", "content" -> input)
      )
    )

    try {
      val client = HttpClient.newBuilder()
        .connectTimeout(Duration.ofSeconds(15))
        .build()

      val request = HttpRequest.newBuilder()
        .uri(URI.create(apiUrl))
        .header("Content-Type", "application/json")
        .header("Authorization", s"Bearer $apiKey")
        .timeout(Duration.ofSeconds(30))
        .POST(HttpRequest.BodyPublishers.ofString(ujson.write(requestBody)))
        .build()

      val response = client.send(request, HttpResponse.BodyHandlers.ofString())

      if (response.statusCode() != 200) {
        return Left(s"API request failed (HTTP ${response.statusCode()}): ${response.body().take(200)}")
      }

      val json = ujson.read(response.body())
      val content = json("choices")(0)("message")("content").str
      parseJsonResponse(content)
    } catch {
      case e: java.net.http.HttpTimeoutException =>
        Left("Request timed out. Please try again.")
      case e: java.net.ConnectException =>
        Left("Could not connect to OpenRouter API. Check your internet connection.")
      case e: Exception =>
        Left(s"Unexpected error: ${e.getMessage}")
    }
  }

  /**
   * Parses the JSON string returned by the LLM into a ParsedInteraction.
   */
  private def parseJsonResponse(content: String): Either[String, ParsedInteraction] = {
    try {
      val json = ujson.read(content)
      def str(key: String): Option[String] =
        json.obj.get(key).flatMap(v => if (v.isNull) None else Some(v.str))

      val personNames = json.obj.get("personNames") match {
        case Some(v) if !v.isNull => v.arr.map(_.str).toList.filter(_.nonEmpty)
        case _ =>
          // Fall back to single personName for backward compatibility
          str("personName").filter(_.nonEmpty).toList
      }
      val medium = str("medium").getOrElse("InPerson")
      val location = str("location").getOrElse("")
      val theirLocation = str("theirLocation")
      val topics = json.obj.get("topics").map(_.arr.map(_.str).toList).getOrElse(List.empty)
      val note = str("note")
      val date = str("date")

      if (personNames.isEmpty) Left("LLM did not extract any person names")
      else if (topics.isEmpty) Left("LLM did not extract any topics")
      else Right(ParsedInteraction(personNames, medium, location, theirLocation, topics, note, date))
    } catch {
      case e: Exception =>
        Left(s"Failed to parse LLM response: ${e.getMessage}")
    }
  }
}
