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
        reviewAndSave(parsed)
    }
  }

  private def reviewAndSave(initial: ParsedInteraction): Unit = {
    var current = initial
    var done = false

    while (!done) {
      displayParsed(current)
      println()
      println("Actions: (s)ave, (e)dit a field, (d)iscard")
      print("Choice: ")
      val choice = StdIn.readLine().trim.toLowerCase

      choice match {
        case "s" | "save" =>
          saveInteraction(current)
          done = true
        case "e" | "edit" =>
          current = editField(current)
        case "d" | "discard" =>
          println("Discarded.")
          done = true
        case _ =>
          println("Invalid choice. Enter 's' to save, 'e' to edit, or 'd' to discard.")
      }
    }
  }

  private def editField(parsed: ParsedInteraction): ParsedInteraction = {
    println()
    println("Which field to edit?")
    println("  1. People")
    println("  2. Medium")
    println("  3. Location")
    println("  4. Their location")
    println("  5. Topics")
    println("  6. Note")
    println("  7. Date")
    print("Field (1-7): ")
    val field = StdIn.readLine().trim

    field match {
      case "1" =>
        print(s"People (comma-separated) [${parsed.personNames.mkString(", ")}]: ")
        val input = StdIn.readLine().trim
        if (input.nonEmpty) {
          val names = input.split(",").map(_.trim).filter(_.nonEmpty).toList
          if (names.nonEmpty) parsed.copy(personNames = names) else parsed
        } else parsed
      case "2" =>
        println("  1. In Person  2. Text  3. Phone Call  4. Video Call  5. Social Media")
        print(s"Medium [${formatMedium(parsed.medium)}]: ")
        val input = StdIn.readLine().trim
        val newMedium = input match {
          case "1" => "InPerson"
          case "2" => "Text"
          case "3" => "PhoneCall"
          case "4" => "VideoCall"
          case "5" => "SocialMedia"
          case "" => parsed.medium
          case _ =>
            println("Invalid selection, keeping current.")
            parsed.medium
        }
        parsed.copy(medium = newMedium)
      case "3" =>
        val current = if (parsed.location.isEmpty) "(not set)" else parsed.location
        print(s"Location [$current]: ")
        val input = StdIn.readLine().trim
        if (input.nonEmpty) parsed.copy(location = input) else parsed
      case "4" =>
        if (parsed.medium == "InPerson") {
          println("Their location is the same as location for in-person interactions.")
          parsed
        } else {
          val current = parsed.theirLocation.getOrElse("(none)")
          print(s"Their location [$current]: ")
          val input = StdIn.readLine().trim
          if (input.nonEmpty) parsed.copy(theirLocation = Some(input))
          else parsed
        }
      case "5" =>
        print(s"Topics (comma-separated) [${parsed.topics.mkString(", ")}]: ")
        val input = StdIn.readLine().trim
        if (input.nonEmpty) {
          val newTopics = input.split(",").map(_.trim).filter(_.nonEmpty).toList
          if (newTopics.nonEmpty) parsed.copy(topics = newTopics) else parsed
        } else parsed
      case "6" =>
        val current = parsed.note.getOrElse("(none)")
        print(s"Note [$current]: ")
        val input = StdIn.readLine().trim
        if (input.nonEmpty) parsed.copy(note = Some(input))
        else parsed
      case "7" =>
        val current = parsed.date.getOrElse("today")
        print(s"Date (YYYY-MM-DD) [$current]: ")
        val input = StdIn.readLine().trim
        if (input.nonEmpty) parsed.copy(date = Some(input))
        else parsed
      case _ =>
        println("Invalid field number.")
        parsed
    }
  }

  private def displayParsed(parsed: ParsedInteraction): Unit = {
    println()
    println("Parsed interaction:")
    println(s"  1. People:         ${parsed.personNames.mkString(", ")}")
    println(s"  2. Medium:         ${formatMedium(parsed.medium)}")
    println(s"  3. Location:       ${if (parsed.location.isEmpty) "(not set)" else parsed.location}")
    if (parsed.medium == "InPerson") {
      println(s"  4. Their location: (same as location)")
    } else {
      println(s"  4. Their location: ${parsed.theirLocation.getOrElse("(not set)")}")
    }
    println(s"  5. Topics:         ${parsed.topics.mkString(", ")}")
    println(s"  6. Note:           ${parsed.note.getOrElse("(none)")}")
    println(s"  7. Date:           ${parsed.date.getOrElse("today")}")
  }

  private def formatMedium(s: String): String = s match {
    case "InPerson"    => "In Person"
    case "Text"        => "Text"
    case "PhoneCall"   => "Phone Call"
    case "VideoCall"   => "Video Call"
    case "SocialMedia" => "Social Media"
    case other         => other
  }

  private def saveInteraction(parsed: ParsedInteraction): Unit = {
    if (parsed.location.isEmpty) {
      println("Error: Location is required. Please edit the location field first.")
      return
    }

    // Resolve all people
    val resolvedPeople = parsed.personNames.flatMap(resolvePerson)
    if (resolvedPeople.isEmpty) {
      println("No people resolved. Interaction not saved.")
      return
    }

    // If some names didn't resolve, confirm saving for the ones that did
    if (resolvedPeople.size < parsed.personNames.size) {
      val resolvedNames = resolvedPeople.map(_.name).mkString(", ")
      print(s"Save interaction for: $resolvedNames? (y/n): ")
      val confirm = StdIn.readLine().trim.toLowerCase
      if (confirm != "y" && confirm != "yes") {
        println("Discarded.")
        return
      }
    }

    val medium = parseMedium(parsed.medium)
    val topics = parsed.topics.toSet
    val note = parsed.note
    val date = parsed.date.flatMap(parseDate).getOrElse(LocalDate.now())

    for (person <- resolvedPeople) {
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

  /**
   * Resolves a person name to a Person in the network.
   * If not found, offers to add them.
   */
  private def resolvePerson(name: String): Option[Person] = {
    val active = NetworkQueries.activePeople(ctx.network)
    val matches = active.filter(p =>
      p.name.toLowerCase.contains(name.toLowerCase) ||
      p.nickname.exists(_.toLowerCase.contains(name.toLowerCase))
    )

    matches match {
      case List(person) => Some(person)
      case multiple if multiple.nonEmpty =>
        multiple.find(_.name.equalsIgnoreCase(name)) match {
          case Some(exact) => Some(exact)
          case None =>
            println(s"Multiple matches for '$name':")
            multiple.foreach(p => println(s"  ${p.name}"))
            println("Please edit the people field to be more specific.")
            None
        }
      case _ =>
        println(s"'$name' is not in your network.")
        print(s"Would you like to add $name? (y/n): ")
        val answer = StdIn.readLine().trim.toLowerCase
        if (answer == "y" || answer == "yes") {
          NetworkOps.addPerson(ctx.network, name) match {
            case Right((updatedNetwork, person)) =>
              ctx.network = updatedNetwork
              ctx.save()
              println(s"Added $name to your network.")
              Some(person)
            case Left(err) =>
              println(s"Error adding person: ${err.message}")
              None
          }
        } else {
          None
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
