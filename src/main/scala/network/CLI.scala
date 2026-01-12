package network

import scala.io.StdIn
import scala.compiletime.uninitialized
import java.nio.file.{Files, Path, Paths}

/**
 * Command-line interface for the personal relationship manager.
 * 
 * Run with --help for usage information.
 */
object CLI {

  val defaultDataDir: Path = Paths.get(System.getProperty("user.home"), ".relationships")
  val defaultDataFile: Path = defaultDataDir.resolve("network.json")

  def main(args: Array[String]): Unit = {
    if (args.contains("--help") || args.contains("-h")) {
      printHelp()
      return
    }

    val dataFile = args
      .sliding(2)
      .collectFirst { case Array("--file", path) => Paths.get(path) }
      .getOrElse(defaultDataFile)

    val repl = new REPL(dataFile)
    repl.run()
  }

  def printHelp(): Unit = {
    println("""
Personal Relationship Manager - CLI

USAGE:
  relationships [OPTIONS]

OPTIONS:
  --help, -h          Show this help message
  --file <path>       Use a custom data file (default: ~/.relationships/network.json)

REPL COMMANDS:
  Once running, type 'help' to see available commands.

GETTING STARTED:
  1. Run the program: sbt run
  2. On first run, you'll be prompted to create your network
  3. Use 'add <n>' to add people
  4. Use 'log <n>' to log interactions
  5. Use 'remind' to see who you should reach out to

DATA STORAGE:
  Your network is saved to ~/.relationships/network.json by default.
  Use --file to specify a different location.
""".trim)
  }
}

/**
 * The interactive REPL for managing your network.
 */
class REPL(dataFile: Path) {

  private var network: Network = uninitialized
  private var user: User = uninitialized
  private var running = true

  def run(): Unit = {
    println("Personal Relationship Manager")
    println("Type 'help' for commands, 'quit' to exit.")
    println()

    loadOrInit()

    while (running) {
      print("> ")
      val input = StdIn.readLine()
      if (input == null) {
        running = false
      } else {
        handleCommand(input.trim)
      }
    }

    println("Goodbye!")
  }

  private def loadOrInit(): Unit = {
    if (Files.exists(dataFile)) {
      JsonCodecs.loadFromFile(dataFile.toString) match {
        case Right(n) =>
          network = n
          user = User(network.ownerId, NetworkQueries.self(network).name, "")
          println(s"Loaded network for ${NetworkQueries.self(network).name}")
          printStats()
        case Left(err) =>
          println(s"Error loading data: $err")
          println("Starting fresh...")
          initNewNetwork()
      }
    } else {
      println("No existing network found.")
      initNewNetwork()
    }
  }

  private def initNewNetwork(): Unit = {
    println()
    print("What's your name? ")
    val name = StdIn.readLine().trim
    if (name.isEmpty) {
      println("Name cannot be empty. Please restart and try again.")
      running = false
      return
    }

    user = User.create(name, "")
    val self = Person.createSelf(name)
    network = Network.create(user, self)
    save()
    println(s"Welcome, $name! Your network has been created.")
    println()
  }

  private def handleCommand(input: String): Unit = {
    val parts = parseInput(input)
    if (parts.isEmpty) return

    val command = parts.head.toLowerCase
    val args = parts.tail

    command match {
      case "help" | "?" => printCommandHelp()
      case "quit" | "exit" | "q" => running = false
      case "list" | "ls" => listPeople()
      case "add" => addPerson(args)
      case "show" | "view" => showPerson(args)
      case "log" => logInteraction(args)
      case "remind" | "reminders" => showReminders()
      case "set-reminder" => setReminder(args)
      case "search" | "find" => searchPeople(args)
      case "edit" => editPerson(args)
      case "edit-labels" => editLabels(args)
      case "archive" => archivePerson(args)
      case "unarchive" => unarchivePerson(args)
      case "archived" => listArchived()
      case "stats" => printStats()
      case "labels" => listLabels()
      case "circles" => listCircles()
      case "add-circle" => addCircle(args)
      case "show-circle" => showCircle(args)
      case "edit-circle" => editCircle(args)
      case "archive-circle" => archiveCircle(args)
      case "unarchive-circle" => unarchiveCircle(args)
      case "archived-circles" => listArchivedCircles()
      case "add-phone" => addPhone(args)
      case "add-email" => addEmail(args)
      case "save" => save(); println("Saved.")
      case "" => // ignore empty
      case _ => println(s"Unknown command: $command. Type 'help' for commands.")
    }
  }

  private def parseInput(input: String): List[String] = {
    val result = scala.collection.mutable.ListBuffer[String]()
    val current = new StringBuilder
    var inQuotes = false

    for (c <- input) {
      c match {
        case '"' => inQuotes = !inQuotes
        case ' ' if !inQuotes =>
          if (current.nonEmpty) {
            result += current.toString
            current.clear()
          }
        case _ => current += c
      }
    }
    if (current.nonEmpty) result += current.toString
    result.toList
  }

  private def printCommandHelp(): Unit = {
    println("""
COMMANDS:

  People:
    list                    List all active people in your network
    add <name>              Add a new person
    show <name>             Show details about a person
    edit <name>             Edit a person's information
    edit-labels <name>      Add/remove labels for a person
    search <query>          Search for people by name
    archive <name>          Archive a person (hide from main list)
    unarchive <name>        Restore an archived person
    archived                List archived people

  Interactions:
    log <name>              Log an interaction with someone
    remind                  Show people you're overdue to contact
    set-reminder <name>     Set reminder frequency for someone

  Contact Info:
    add-phone <name>        Add a phone number to someone
    add-email <name>        Add an email address to someone

  Organization:
    labels                  List all relationship labels
    circles                 List all active circles
    add-circle <name>       Create a new circle
    show-circle <name>      Show circle details and members
    edit-circle <name>      Edit circle name, members
    archive-circle <name>   Archive a circle
    unarchive-circle <name> Restore an archived circle
    archived-circles        List archived circles

  Other:
    stats                   Show network statistics
    save                    Manually save (auto-saves after changes)
    help                    Show this help
    exit or quit            Exit the program

TIPS:
  - Names are matched case-insensitively
  - Partial name matches work for most commands
""".trim)
  }

  // ==========================================================================
  // PEOPLE COMMANDS
  // ==========================================================================

  private def listPeople(): Unit = {
    val people = NetworkQueries.activePeople(network)
    if (people.isEmpty) {
      println("No people in your network yet. Use 'add <name>' to add someone.")
      return
    }

    println(s"People in your network (${people.size}):")
    println()
    for (person <- people) {
      val labels = NetworkQueries.labelsFor(network, person.id).map(_.name).toList.sorted
      val labelStr = if (labels.nonEmpty) s" [${labels.mkString(", ")}]" else ""
      val lastContact = NetworkQueries.daysSinceInteraction(network, person.id)
        .map(d => s" - last contact: ${formatDaysAgo(d)}")
        .getOrElse("")
      println(s"  ${person.name}$labelStr$lastContact")
    }
  }

  private def addPerson(args: List[String]): Unit = {
    val name = if (args.nonEmpty) args.mkString(" ") else {
      print("Name: ")
      StdIn.readLine().trim
    }

    if (name.isEmpty) {
      println("Name cannot be empty.")
      return
    }

    NetworkOps.addPerson(network, name) match {
      case Right((updated, person)) =>
        network = updated
        save()
        println(s"Added ${person.name}")

        print("How did you meet? (press Enter to skip) ")
        val howWeMet = StdIn.readLine().trim
        if (howWeMet.nonEmpty) {
          NetworkOps.updatePerson(network, person.id, howWeMet = Some(Some(howWeMet))) match {
            case Right(n) => network = n; save()
            case Left(_) => // ignore
          }
        }

        println("Labels (enter numbers separated by spaces, or press Enter to skip):")
        val labels = network.relationshipLabels.values.toList.sortBy(_.name)
        labels.zipWithIndex.foreach { case (label, i) =>
          println(s"  ${i + 1}. ${label.name}")
        }
        print("Labels: ")
        val labelInput = StdIn.readLine().trim
        if (labelInput.nonEmpty) {
          val indices = labelInput.split("\\s+").flatMap(s => scala.util.Try(s.toInt - 1).toOption)
          val selectedLabels = indices.flatMap(i => labels.lift(i)).map(_.id).toSet
          if (selectedLabels.nonEmpty) {
            NetworkOps.setRelationship(network, person.id, selectedLabels) match {
              case Right(n) => network = n; save()
              case Left(_) => // ignore
            }
          }
        }

        print("Reminder every how many days? (press Enter to skip) ")
        val reminderInput = StdIn.readLine().trim
        if (reminderInput.nonEmpty) {
          scala.util.Try(reminderInput.toInt).toOption match {
            case Some(days) if days > 0 =>
              val rel = network.relationships.get(person.id)
              if (rel.isDefined) {
                NetworkOps.setReminder(network, person.id, Some(days)) match {
                  case Right(n) => network = n; save(); println(s"Reminder set for every $days days")
                  case Left(e) => println(s"Error: ${e.message}")
                }
              } else {
                NetworkOps.setRelationship(network, person.id, reminderDays = Some(days)) match {
                  case Right(n) => network = n; save(); println(s"Reminder set for every $days days")
                  case Left(e) => println(s"Error: ${e.message}")
                }
              }
            case _ => // ignore invalid input
          }
        }

      case Left(err) =>
        println(s"Error: ${err.message}")
    }
  }

  private def showPerson(args: List[String]): Unit = {
    findPerson(args) match {
      case Some(person) =>
        println()
        println(s"Name: ${person.name}")
        person.nickname.foreach(n => println(s"Nickname: $n"))
        person.howWeMet.foreach(h => println(s"How we met: $h"))
        person.birthday.foreach(b => println(s"Birthday: $b"))
        person.notes.foreach(n => println(s"Notes: $n"))
        person.defaultLocation.foreach(l => println(s"Default location: $l"))

        val labels = NetworkQueries.labelsFor(network, person.id)
        if (labels.nonEmpty) {
          println(s"Labels: ${labels.map(_.name).toList.sorted.mkString(", ")}")
        }

        val circles = NetworkQueries.circlesFor(network, person.id)
        if (circles.nonEmpty) {
          println(s"Circles: ${circles.map(_.name).mkString(", ")}")
        }

        val contacts = NetworkQueries.allContactsFor(network, person.id)
        if (contacts.nonEmpty) {
          println("Contact info:")
          for (entry <- contacts) {
            val label = entry.label.map(l => s" ($l)").getOrElse("")
            val value = entry.value match {
              case ContactValue.StringValue(v) => v
              case ContactValue.AddressValue(a) => s"${a.street}, ${a.city}, ${a.state} ${a.zip}, ${a.country}"
            }
            val typeName = entry.contactType match {
              case ContactType.Phone => "Phone"
              case ContactType.Email => "Email"
              case ContactType.PhysicalAddress => "Address"
              case ContactType.Custom(typeId) => 
                NetworkQueries.customContactTypeName(network, typeId).getOrElse("Custom")
            }
            println(s"  $typeName$label: $value")
          }
        }

        network.relationships.get(person.id).flatMap(_.reminderDays).foreach { days =>
          println(s"Reminder: every $days days")
        }

        NetworkQueries.lastInteractionWith(network, person.id) match {
          case Some(interaction) =>
            val daysAgo = NetworkQueries.daysSinceInteraction(network, person.id).getOrElse(0L)
            val mediumStr = InteractionMedium.name(interaction.medium)
            println(s"Last interaction: ${formatDaysAgo(daysAgo)} via $mediumStr")
            println(s"  My location: ${interaction.myLocation}")
            interaction.theirLocation.foreach(loc => 
              if (interaction.medium != InteractionMedium.InPerson || loc != interaction.myLocation) {
                println(s"  Their location: $loc")
              }
            )
            println(s"  Topics: ${interaction.topics.mkString(", ")}")
            interaction.note.foreach(n => println(s"  Note: $n"))
          case None =>
            println("No interactions logged yet")
        }

        val interactions = NetworkQueries.interactionsWith(network, person.id)
        if (interactions.size > 1) {
          println(s"Total interactions: ${interactions.size}")
        }
        println()

      case None =>
        if (args.isEmpty) println("Usage: show <name>")
    }
  }

  private def logInteraction(args: List[String]): Unit = {
    findPerson(args) match {
      case Some(person) =>
        println(s"Logging interaction with ${person.name}")
        
        println("How did you interact?")
        InteractionMedium.all.zipWithIndex.foreach { case (medium, i) =>
          println(s"  ${i + 1}. ${InteractionMedium.name(medium)}")
        }
        print("Medium (1-5): ")
        val mediumInput = StdIn.readLine().trim
        val medium = scala.util.Try(mediumInput.toInt - 1).toOption
          .flatMap(i => InteractionMedium.all.lift(i))
          .getOrElse {
            println("Invalid selection, defaulting to In Person")
            InteractionMedium.InPerson
          }

        val (myLocation, theirLocation) = if (medium == InteractionMedium.InPerson) {
          val defaultLoc = person.defaultLocation.map(l => s" [$l]").getOrElse("")
          print(s"Location$defaultLoc: ")
          val locInput = StdIn.readLine().trim
          val loc = if (locInput.isEmpty) person.defaultLocation.getOrElse("") else locInput
          if (loc.isEmpty) {
            println("Location is required.")
            return
          }
          (loc, Some(loc))
        } else {
          print("Your location: ")
          val myLoc = StdIn.readLine().trim
          if (myLoc.isEmpty) {
            println("Your location is required.")
            return
          }
          
          val defaultLoc = person.defaultLocation.map(l => s" [$l]").getOrElse("")
          print(s"Their location (optional)$defaultLoc: ")
          val theirLocInput = StdIn.readLine().trim
          val theirLoc = if (theirLocInput.isEmpty) person.defaultLocation else Some(theirLocInput)
          (myLoc, theirLoc)
        }

        print("Topics (comma-separated): ")
        val topicsInput = StdIn.readLine().trim
        val topics = topicsInput.split(",").map(_.trim).filter(_.nonEmpty).toSet
        if (topics.isEmpty) {
          println("At least one topic is required.")
          return
        }

        print("Note (optional): ")
        val note = StdIn.readLine().trim
        val noteOpt = if (note.isEmpty) None else Some(note)

        val result = if (medium == InteractionMedium.InPerson) {
          NetworkOps.logInPersonInteraction(network, person.id, myLocation, topics, noteOpt)
        } else {
          NetworkOps.logRemoteInteraction(network, person.id, medium, myLocation, theirLocation, topics, noteOpt)
        }

        result match {
          case Right(updated) =>
            network = updated
            save()
            println(s"Logged interaction with ${person.name}")
          case Left(err) =>
            println(s"Error: ${err.message}")
        }

      case None =>
        if (args.isEmpty) println("Usage: log <name>")
    }
  }

  private def showReminders(): Unit = {
    val overdue = NetworkQueries.peopleNeedingReminder(network)
    if (overdue.isEmpty) {
      println("No overdue reminders! You're all caught up.")
      return
    }

    println(s"People to reach out to (${overdue.size}):")
    println()
    for (status <- overdue) {
      val overdueStr = status.overdueStatus match {
        case OverdueStatus.NeverContacted => 
          "never contacted"
        case OverdueStatus.DaysOverdue(days) =>
          val lastContact = status.daysSinceLastInteraction
            .map(d => s"last contact ${formatDaysAgo(d)}")
            .getOrElse("")
          s"$days days overdue ($lastContact)"
      }
      println(s"  ${status.person.name} - $overdueStr")
    }
  }

  private def setReminder(args: List[String]): Unit = {
    findPerson(args) match {
      case Some(person) =>
        print(s"Remind every how many days? (0 to remove reminder): ")
        val input = StdIn.readLine().trim
        scala.util.Try(input.toInt).toOption match {
          case Some(0) =>
            NetworkOps.setReminder(network, person.id, None) match {
              case Right(n) =>
                network = n
                save()
                println(s"Reminder removed for ${person.name}")
              case Left(_) =>
                println(s"No reminder was set for ${person.name}")
            }
          case Some(days) if days > 0 =>
            val updated = if (!network.relationships.contains(person.id)) {
              NetworkOps.setRelationship(network, person.id, reminderDays = Some(days))
            } else {
              NetworkOps.setReminder(network, person.id, Some(days))
            }
            updated match {
              case Right(n) =>
                network = n
                save()
                println(s"Reminder set: reach out to ${person.name} every $days days")
              case Left(e) =>
                println(s"Error: ${e.message}")
            }
          case _ =>
            println("Invalid number")
        }

      case None =>
        if (args.isEmpty) println("Usage: set-reminder <name>")
    }
  }

  private def searchPeople(args: List[String]): Unit = {
    if (args.isEmpty) {
      println("Usage: search <query>")
      return
    }

    val query = args.mkString(" ")
    val results = NetworkQueries.findByName(network, query).filter(!_.isSelf)
    
    if (results.isEmpty) {
      println(s"No people found matching '$query'")
    } else {
      println(s"Found ${results.size} match(es):")
      for (person <- results) {
        val archived = if (person.archived) " (archived)" else ""
        println(s"  ${person.name}$archived")
      }
    }
  }

  private def editPerson(args: List[String]): Unit = {
    findPerson(args) match {
      case Some(person) =>
        println(s"Editing ${person.name} (press Enter to keep current value)")
        println()

        print(s"Name [${person.name}]: ")
        val nameInput = StdIn.readLine().trim
        val newName = if (nameInput.isEmpty) None else Some(nameInput)

        print(s"Nickname [${person.nickname.getOrElse("")}]: ")
        val nicknameInput = StdIn.readLine().trim
        val newNickname = if (nicknameInput.isEmpty) None else Some(Some(nicknameInput))

        print(s"How we met [${person.howWeMet.getOrElse("")}]: ")
        val howWeMetInput = StdIn.readLine().trim
        val newHowWeMet = if (howWeMetInput.isEmpty) None else Some(Some(howWeMetInput))

        print(s"Notes [${person.notes.getOrElse("")}]: ")
        val notesInput = StdIn.readLine().trim
        val newNotes = if (notesInput.isEmpty) None else Some(Some(notesInput))

        print(s"Default location [${person.defaultLocation.getOrElse("")}]: ")
        val locInput = StdIn.readLine().trim
        val newLoc = if (locInput.isEmpty) None else Some(Some(locInput))

        NetworkOps.updatePerson(
          network, person.id,
          name = newName,
          nickname = newNickname,
          howWeMet = newHowWeMet,
          notes = newNotes,
          defaultLocation = newLoc
        ) match {
          case Right(n) =>
            network = n
            save()
            println(s"Updated ${newName.getOrElse(person.name)}")
          case Left(e) =>
            println(s"Error: ${e.message}")
        }

      case None =>
        if (args.isEmpty) println("Usage: edit <name>")
    }
  }

  private def editLabels(args: List[String]): Unit = {
    findPerson(args) match {
      case Some(person) =>
        val currentLabels = NetworkQueries.labelsFor(network, person.id)
        val allLabels = network.relationshipLabels.values.toList.sortBy(_.name)
        
        println(s"Editing labels for ${person.name}")
        println()
        println("Current labels: " + (if (currentLabels.isEmpty) "(none)" else currentLabels.map(_.name).toList.sorted.mkString(", ")))
        println()
        println("Available labels (enter numbers to toggle, press Enter when done):")
        
        var selectedIds = currentLabels.map(_.id)
        var editing = true
        
        while (editing) {
          allLabels.zipWithIndex.foreach { case (label, i) =>
            val marker = if (selectedIds.contains(label.id)) "[x]" else "[ ]"
            println(s"  ${i + 1}. $marker ${label.name}")
          }
          print("Toggle (or Enter to finish): ")
          val input = StdIn.readLine().trim
          
          if (input.isEmpty) {
            editing = false
          } else {
            input.split("\\s+").foreach { s =>
              scala.util.Try(s.toInt - 1).toOption.flatMap(i => allLabels.lift(i)).foreach { label =>
                if (selectedIds.contains(label.id)) {
                  selectedIds = selectedIds - label.id
                } else {
                  selectedIds = selectedIds + label.id
                }
              }
            }
          }
        }
        
        NetworkOps.setLabels(network, person.id, selectedIds) match {
          case Right(n) =>
            network = n
            save()
            val newLabels = selectedIds.flatMap(network.relationshipLabels.get).map(_.name).toList.sorted
            println(s"Labels updated: ${if (newLabels.isEmpty) "(none)" else newLabels.mkString(", ")}")
          case Left(e) =>
            println(s"Error: ${e.message}")
        }

      case None =>
        if (args.isEmpty) println("Usage: edit-labels <name>")
    }
  }

  private def archivePerson(args: List[String]): Unit = {
    findPerson(args) match {
      case Some(person) =>
        NetworkOps.archivePerson(network, person.id) match {
          case Right(n) =>
            network = n
            save()
            println(s"Archived ${person.name}")
          case Left(e) =>
            println(s"Error: ${e.message}")
        }
      case None =>
        if (args.isEmpty) println("Usage: archive <name>")
    }
  }

  private def unarchivePerson(args: List[String]): Unit = {
    val query = args.mkString(" ")
    if (query.isEmpty) {
      println("Usage: unarchive <name>")
      return
    }

    val archived = NetworkQueries.archivedPeople(network)
    val matches = archived.filter(p => 
      p.name.toLowerCase.contains(query.toLowerCase) ||
      p.nickname.exists(_.toLowerCase.contains(query.toLowerCase))
    )

    matches match {
      case Nil => println(s"No archived person found matching '$query'")
      case List(person) =>
        NetworkOps.unarchivePerson(network, person.id) match {
          case Right(n) =>
            network = n
            save()
            println(s"Restored ${person.name}")
          case Left(e) =>
            println(s"Error: ${e.message}")
        }
      case multiple =>
        println("Multiple matches found:")
        multiple.foreach(p => println(s"  ${p.name}"))
        println("Please be more specific.")
    }
  }

  private def listArchived(): Unit = {
    val archived = NetworkQueries.archivedPeople(network)
    if (archived.isEmpty) {
      println("No archived people.")
    } else {
      println(s"Archived people (${archived.size}):")
      archived.foreach(p => println(s"  ${p.name}"))
    }
  }

  // ==========================================================================
  // CIRCLE COMMANDS
  // ==========================================================================

  private def listCircles(): Unit = {
    val circles = NetworkQueries.activeCircles(network)
    if (circles.isEmpty) {
      println("No circles yet. Use 'add-circle <name>' to create one.")
    } else {
      println(s"Circles (${circles.size}):")
      for (circle <- circles) {
        val count = circle.memberIds.size
        println(s"  ${circle.name} ($count members)")
      }
    }
  }

  private def addCircle(args: List[String]): Unit = {
    val name = if (args.nonEmpty) args.mkString(" ") else {
      print("Circle name: ")
      StdIn.readLine().trim
    }

    if (name.isEmpty) {
      println("Name cannot be empty.")
      return
    }

    println("Add members (enter numbers separated by spaces, or press Enter to skip):")
    val people = NetworkQueries.activePeople(network)
    people.zipWithIndex.foreach { case (person, i) =>
      println(s"  ${i + 1}. ${person.name}")
    }
    print("Members: ")
    val memberInput = StdIn.readLine().trim
    val memberIds = if (memberInput.isEmpty) {
      Set.empty[Id[Person]]
    } else {
      memberInput.split("\\s+")
        .flatMap(s => scala.util.Try(s.toInt - 1).toOption)
        .flatMap(i => people.lift(i))
        .map(_.id)
        .toSet
    }

    NetworkOps.createCircle(network, name, memberIds = memberIds) match {
      case Right((updated, circle)) =>
        network = updated
        save()
        println(s"Created circle: ${circle.name} with ${memberIds.size} members")
      case Left(err) =>
        println(s"Error: ${err.message}")
    }
  }

  private def showCircle(args: List[String]): Unit = {
    findCircle(args) match {
      case Some(circle) =>
        println()
        println(s"Circle: ${circle.name}")
        circle.description.foreach(d => println(s"Description: $d"))
        if (circle.archived) println("(archived)")
        
        val members = NetworkQueries.circleMembers(network, circle.id)
        if (members.isEmpty) {
          println("No members")
        } else {
          println(s"Members (${members.size}):")
          members.foreach(p => println(s"  ${p.name}"))
        }
        println()
      case None =>
        if (args.isEmpty) println("Usage: show-circle <name>")
    }
  }

  private def editCircle(args: List[String]): Unit = {
    findCircle(args) match {
      case Some(circle) =>
        println(s"Editing circle: ${circle.name}")
        println()

        print(s"Name [${circle.name}]: ")
        val nameInput = StdIn.readLine().trim
        if (nameInput.nonEmpty) {
          NetworkOps.updateCircle(network, circle.id, name = Some(nameInput)) match {
            case Right(n) => network = n; save()
            case Left(e) => println(s"Error: ${e.message}")
          }
        }

        val currentMembers = NetworkQueries.circleMembers(network, circle.id).map(_.id).toSet
        val allPeople = NetworkQueries.activePeople(network)
        
        println()
        println("Edit members (enter numbers to toggle, press Enter when done):")
        
        var selectedIds = currentMembers
        var editing = true
        
        while (editing) {
          allPeople.zipWithIndex.foreach { case (person, i) =>
            val marker = if (selectedIds.contains(person.id)) "[x]" else "[ ]"
            println(s"  ${i + 1}. $marker ${person.name}")
          }
          print("Toggle (or Enter to finish): ")
          val input = StdIn.readLine().trim
          
          if (input.isEmpty) {
            editing = false
          } else {
            input.split("\\s+").foreach { s =>
              scala.util.Try(s.toInt - 1).toOption.flatMap(i => allPeople.lift(i)).foreach { person =>
                if (selectedIds.contains(person.id)) {
                  selectedIds = selectedIds - person.id
                } else {
                  selectedIds = selectedIds + person.id
                }
              }
            }
          }
        }
        
        NetworkOps.setCircleMembers(network, circle.id, selectedIds) match {
          case Right(n) =>
            network = n
            save()
            println(s"Circle updated with ${selectedIds.size} members")
          case Left(e) =>
            println(s"Error: ${e.message}")
        }

      case None =>
        if (args.isEmpty) println("Usage: edit-circle <name>")
    }
  }

  private def archiveCircle(args: List[String]): Unit = {
    findCircle(args) match {
      case Some(circle) =>
        NetworkOps.archiveCircle(network, circle.id) match {
          case Right(n) =>
            network = n
            save()
            println(s"Archived circle: ${circle.name}")
          case Left(e) =>
            println(s"Error: ${e.message}")
        }
      case None =>
        if (args.isEmpty) println("Usage: archive-circle <name>")
    }
  }

  private def unarchiveCircle(args: List[String]): Unit = {
    val query = args.mkString(" ")
    if (query.isEmpty) {
      println("Usage: unarchive-circle <name>")
      return
    }

    val archived = NetworkQueries.archivedCircles(network)
    val matches = archived.filter(_.name.toLowerCase.contains(query.toLowerCase))

    matches match {
      case Nil => println(s"No archived circle found matching '$query'")
      case List(circle) =>
        NetworkOps.unarchiveCircle(network, circle.id) match {
          case Right(n) =>
            network = n
            save()
            println(s"Restored circle: ${circle.name}")
          case Left(e) =>
            println(s"Error: ${e.message}")
        }
      case multiple =>
        println("Multiple matches found:")
        multiple.foreach(c => println(s"  ${c.name}"))
        println("Please be more specific.")
    }
  }

  private def listArchivedCircles(): Unit = {
    val archived = NetworkQueries.archivedCircles(network)
    if (archived.isEmpty) {
      println("No archived circles.")
    } else {
      println(s"Archived circles (${archived.size}):")
      archived.foreach(c => println(s"  ${c.name}"))
    }
  }

  // ==========================================================================
  // LABELS AND CONTACT INFO
  // ==========================================================================

  private def listLabels(): Unit = {
    val labels = network.relationshipLabels.values.toList.sortBy(_.name)
    println(s"Relationship labels (${labels.size}):")
    for (label <- labels) {
      val count = NetworkQueries.peopleWithLabel(network, label.id).size
      println(s"  ${label.name} $count")
    }
  }

  private def addPhone(args: List[String]): Unit = {
    findPerson(args) match {
      case Some(person) =>
        print("Phone number: ")
        val number = StdIn.readLine().trim
        if (number.isEmpty) {
          println("Phone number cannot be empty.")
          return
        }

        print("Label (e.g., work, personal) - optional: ")
        val label = StdIn.readLine().trim
        val labelOpt = if (label.isEmpty) None else Some(label)

        NetworkOps.addPhone(network, person.id, number, labelOpt) match {
          case Right(n) =>
            network = n
            save()
            println(s"Added phone number for ${person.name}")
          case Left(e) =>
            println(s"Error: ${e.message}")
        }

      case None =>
        if (args.isEmpty) println("Usage: add-phone <name>")
    }
  }

  private def addEmail(args: List[String]): Unit = {
    findPerson(args) match {
      case Some(person) =>
        print("Email address: ")
        val email = StdIn.readLine().trim
        if (email.isEmpty) {
          println("Email cannot be empty.")
          return
        }

        print("Label (e.g., work, personal) - optional: ")
        val label = StdIn.readLine().trim
        val labelOpt = if (label.isEmpty) None else Some(label)

        NetworkOps.addEmail(network, person.id, email, labelOpt) match {
          case Right(n) =>
            network = n
            save()
            println(s"Added email for ${person.name}")
          case Left(e) =>
            println(s"Error: ${e.message}")
        }

      case None =>
        if (args.isEmpty) println("Usage: add-email <name>")
    }
  }

  // ==========================================================================
  // STATS
  // ==========================================================================

  private def printStats(): Unit = {
    val s = NetworkQueries.stats(network)
    println()
    println(s"People: ${s.activePeople} active, ${s.archivedPeople} archived")
    println(s"Interactions: ${s.totalInteractions}")
    println(s"Circles: ${s.activeCircles} active, ${s.archivedCircles} archived")
    if (s.remindersOverdue > 0) {
      println(s"Reminders overdue: ${s.remindersOverdue}")
    }
    println()
  }

  // ==========================================================================
  // HELPERS
  // ==========================================================================

  private def findPerson(args: List[String]): Option[Person] = {
    val query = args.mkString(" ")
    if (query.isEmpty) return None

    val active = NetworkQueries.activePeople(network)
    val matches = active.filter(p =>
      p.name.toLowerCase.contains(query.toLowerCase) ||
      p.nickname.exists(_.toLowerCase.contains(query.toLowerCase))
    )

    matches match {
      case Nil =>
        println(s"No person found matching '$query'")
        None
      case List(person) =>
        Some(person)
      case multiple =>
        multiple.find(_.name.equalsIgnoreCase(query)) match {
          case Some(exact) => Some(exact)
          case None =>
            println("Multiple matches found:")
            multiple.foreach(p => println(s"  ${p.name}"))
            println("Please be more specific.")
            None
        }
    }
  }

  private def findCircle(args: List[String]): Option[Circle] = {
    val query = args.mkString(" ")
    if (query.isEmpty) return None

    val matches = NetworkQueries.findActiveCircleByName(network, query)

    matches match {
      case Nil =>
        println(s"No circle found matching '$query'")
        None
      case List(circle) =>
        Some(circle)
      case multiple =>
        multiple.find(_.name.equalsIgnoreCase(query)) match {
          case Some(exact) => Some(exact)
          case None =>
            println("Multiple matches found:")
            multiple.foreach(c => println(s"  ${c.name}"))
            println("Please be more specific.")
            None
        }
    }
  }

  private def formatDaysAgo(days: Long): String = {
    days match {
      case 0 => "today"
      case 1 => "yesterday"
      case n if n < 7 => s"$n days ago"
      case n if n < 30 => s"${n / 7} week(s) ago"
      case n if n < 365 => s"${n / 30} month(s) ago"
      case n => s"${n / 365} year(s) ago"
    }
  }

  private def save(): Unit = {
    if (!Files.exists(dataFile.getParent)) {
      Files.createDirectories(dataFile.getParent)
    }

    JsonCodecs.saveToFile(network, dataFile.toString) match {
      case Right(_) => // success
      case Left(err) => println(s"Warning: Failed to save: $err")
    }
  }
}
