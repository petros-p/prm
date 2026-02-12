package network

import scala.io.StdIn
import java.time.LocalDate
import java.time.format.DateTimeParseException

/**
 * CLI command handler for AI-assisted interaction logging.
 * Accepts a natural language description and uses an LLM to extract structured fields.
 */
class AILogCommand(ctx: CLIContext) {

  /**
   * Parses a natural language interaction description via LLM and saves after user review.
   * Usage: ai-log Had coffee with John at Starbucks, talked about his new job
   */
  def log(args: List[String]): Unit = {
    if (args.isEmpty) {
      println("Usage: ai-log <natural language description>")
      println("Example: ai-log Had coffee with John at Starbucks, talked about his new job")
      return
    }

    val apiKey = LLMService.getApiKey match {
      case Right(key) => key
      case Left(err) =>
        println(s"Error: $err")
        return
    }

    val input = args.mkString(" ")
    val knownNames = NetworkQueries.activePeople(ctx.network)
      .filterNot(_.isSelf)
      .map(_.name)

    println("Parsing with AI...")
    LLMService.parseInteraction(input, knownNames, apiKey) match {
      case Left(err) =>
        println(s"Error: $err")
      case Right(parsed) =>
        displayParsed(parsed)

        print("\nSave this interaction? (y/n): ")
        val confirm = StdIn.readLine().trim.toLowerCase
        if (confirm == "y" || confirm == "yes") {
          saveInteraction(parsed)
        } else {
          println("Discarded.")
        }
    }
  }

  private def displayParsed(parsed: ParsedInteraction): Unit = {
    println()
    println("Parsed interaction:")
    println(s"  Person:   ${parsed.personName}")
    println(s"  Medium:   ${parsed.medium}")
    println(s"  Location: ${parsed.location}")
    parsed.theirLocation.foreach(l => println(s"  Their location: $l"))
    println(s"  Topics:   ${parsed.topics.mkString(", ")}")
    parsed.note.foreach(n => println(s"  Note:     $n"))
    val dateStr = parsed.date.getOrElse("today")
    println(s"  Date:     $dateStr")
  }

  private def saveInteraction(parsed: ParsedInteraction): Unit = {
    // Resolve person
    val personOpt = ctx.findPerson(List(parsed.personName))
    personOpt match {
      case None => return
      case Some(person) =>
        val medium = parseMedium(parsed.medium)
        val topics = parsed.topics.toSet
        val note = parsed.note
        val date = parsed.date.flatMap(parseDate).getOrElse(LocalDate.now())

        val result = if (medium == InteractionMedium.InPerson) {
          NetworkOps.logInPersonInteraction(ctx.network, person.id, parsed.location, topics, note, date)
        } else {
          NetworkOps.logRemoteInteraction(ctx.network, person.id, medium, parsed.location, parsed.theirLocation, topics, note, date)
        }

        ctx.withSave(result) {
          println(s"Logged interaction with ${person.name}")
        }
    }
  }

  private def parseMedium(s: String): InteractionMedium = s match {
    case "InPerson"    => InteractionMedium.InPerson
    case "Text"        => InteractionMedium.Text
    case "PhoneCall"   => InteractionMedium.PhoneCall
    case "VideoCall"   => InteractionMedium.VideoCall
    case "SocialMedia" => InteractionMedium.SocialMedia
    case _ =>
      println(s"Unknown medium '$s', defaulting to In Person")
      InteractionMedium.InPerson
  }

  private def parseDate(s: String): Option[LocalDate] = {
    try {
      Some(LocalDate.parse(s))
    } catch {
      case _: DateTimeParseException =>
        println(s"Could not parse date '$s', using today")
        None
    }
  }
}
