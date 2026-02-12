package network

import java.net.URI
import java.net.http.{HttpClient, HttpRequest, HttpResponse}
import java.time.Duration
import ujson._

/**
 * Parsed interaction fields extracted by the LLM from natural language input.
 */
case class ParsedInteraction(
  personName: String,
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
  private val model = "meta-llama/llama-3.3-70b-instruct:free"

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
    val systemPrompt =
      s"""You extract interaction metadata from natural language descriptions.
         |Known contacts: [$namesStr]
         |Respond with JSON only, no other text.
         |JSON schema: { "personName": "...", "medium": "InPerson|Text|PhoneCall|VideoCall|SocialMedia", "location": "...", "theirLocation": null, "topics": ["..."], "note": null, "date": null }
         |Rules:
         |- personName must match one of the known contacts as closely as possible
         |- If the medium is not mentioned, default to "InPerson"
         |- location is where the interaction happened (required, use a reasonable default from context)
         |- theirLocation is only for remote interactions where their location differs; set to null for in-person
         |- topics should be a list of discussion subjects extracted from the description
         |- note is for any additional context not captured in other fields; set to null if none
         |- date should be "YYYY-MM-DD" if mentioned, otherwise null (today will be used)""".stripMargin

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
      val personName = json("personName").str
      val medium = json("medium").str
      val location = json("location").str
      val theirLocation = json.obj.get("theirLocation").flatMap(v => if (v.isNull) None else Some(v.str))
      val topics = json("topics").arr.map(_.str).toList
      val note = json.obj.get("note").flatMap(v => if (v.isNull) None else Some(v.str))
      val date = json.obj.get("date").flatMap(v => if (v.isNull) None else Some(v.str))

      if (personName.isEmpty) Left("LLM did not extract a person name")
      else if (topics.isEmpty) Left("LLM did not extract any topics")
      else Right(ParsedInteraction(personName, medium, location, theirLocation, topics, note, date))
    } catch {
      case e: Exception =>
        Left(s"Failed to parse LLM response: ${e.getMessage}")
    }
  }
}
