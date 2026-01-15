package network

import scala.io.StdIn
import scala.compiletime.uninitialized
import java.nio.file.{Files, Path, Paths}

/**
 * Command-line interface for the personal relationship manager.
 */
object CLI {

  val defaultDataDir: Path = Paths.get(".data")
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
Personal Relationship Manager

USAGE:
  sbt run [OPTIONS]

OPTIONS:
  --help, -h      Show this help
  --file <path>   Use custom data file (default: .data/network.json)

Run without options to start the interactive shell.
""".trim)
  }
}

/**
 * Shared context for CLI commands. Holds mutable state and helper methods.
 */
class CLIContext(val dataFile: Path) {
  var network: Network = uninitialized
  var user: User = uninitialized

  def withSave(result: Either[ValidationError, Network])(onSuccess: => Unit): Unit =
    result match {
      case Right(n) =>
        network = n
        save()
        onSuccess
      case Left(e) =>
        println(s"Error: ${e.message}")
    }

  def withSaveAndResult[A](result: Either[ValidationError, (Network, A)])(onSuccess: A => Unit): Unit =
    result match {
      case Right((n, a)) =>
        network = n
        save()
        onSuccess(a)
      case Left(e) =>
        println(s"Error: ${e.message}")
    }

  def save(): Unit = {
    if (!Files.exists(dataFile.getParent)) {
      Files.createDirectories(dataFile.getParent)
    }
    JsonCodecs.saveToFile(network, dataFile.toString) match {
      case Right(_) => // success
      case Left(err) => println(s"Warning: Failed to save: $err")
    }
  }

  def findPerson(args: List[String]): Option[Person] = {
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

  def findCircle(args: List[String]): Option[Circle] = {
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

  def findLabel(args: List[String]): Option[RelationshipLabel] = {
    val query = args.mkString(" ")
    if (query.isEmpty) return None

    val labels = network.relationshipLabels.values.toList
    val matches = labels.filter(_.name.toLowerCase.contains(query.toLowerCase))

    matches match {
      case Nil =>
        println(s"No label found matching '$query'")
        None
      case List(label) =>
        Some(label)
      case multiple =>
        multiple.find(_.name.equalsIgnoreCase(query)) match {
          case Some(exact) => Some(exact)
          case None =>
            println("Multiple matches found:")
            multiple.foreach(l => println(s"  ${l.name}"))
            println("Please be more specific.")
            None
        }
    }
  }

  def formatDaysAgo(days: Long): String = days match {
    case 0 => "today"
    case 1 => "yesterday"
    case n if n < 7 => s"$n days ago"
    case n if n < 30 => s"${n / 7} week(s) ago"
    case n if n < 365 => s"${n / 30} month(s) ago"
    case n => s"${n / 365} year(s) ago"
  }
}

/**
 * The interactive REPL.
 */
class REPL(dataFile: Path) {
  private val ctx = new CLIContext(dataFile)
  private val personCommands = new PersonCommands(ctx)
  private val circleCommands = new CircleCommands(ctx)
  private val interactionCommands = new InteractionCommands(ctx)
  private var running = true

  def run(): Unit = {
    println("Personal Relationship Manager")
    println("Type 'help' for commands, 'exit' to exit.")
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
  }

  private def loadOrInit(): Unit = {
    if (Files.exists(dataFile)) {
      JsonCodecs.loadFromFile(dataFile.toString) match {
        case Right(n) =>
          ctx.network = n
          ctx.user = User(ctx.network.ownerId, NetworkQueries.self(ctx.network).name, "")
          println(s"Loaded network for ${NetworkQueries.self(ctx.network).name}")
          interactionCommands.printStats()
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
    val nameLower = name.toLowerCase

    if (nameLower == "exit" || nameLower == "quit" || nameLower == "q") {
      running = false
      return
    }

    if (name.isEmpty) {
      println("Name cannot be empty. Please restart and try again.")
      running = false
      return
    }

    ctx.user = User.create(name, "")
    val self = Person.createSelf(name)
    ctx.network = Network.create(ctx.user, self)

    // Assign "me" label to self
    val meLabel = ctx.network.relationshipLabels.values.find(_.name == "me")
    meLabel.foreach { label =>
      NetworkOps.setRelationship(ctx.network, self.id, Set(label.id)) match {
        case Right(n) => ctx.network = n
        case Left(e) => println(s"Error: ${e.message}")
      }
    }

    ctx.save()
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

      // Person commands
      case "list" | "ls" => personCommands.list()
      case "add" => personCommands.add(args)
      case "show-person" | "show" | "view" => personCommands.show(args)
      case "edit" => personCommands.edit(args)
      case "search" | "find" => personCommands.search(args)
      case "archive" => personCommands.archive(args)
      case "unarchive" => personCommands.unarchive(args)
      case "archived" => personCommands.listArchived()

      // Circle commands
      case "circles" => circleCommands.list()
      case "add-circle" => circleCommands.add(args)
      case "show-circle" => circleCommands.show(args)
      case "edit-circle" => circleCommands.edit(args)
      case "archive-circle" => circleCommands.archive(args)
      case "unarchive-circle" => circleCommands.unarchive(args)
      case "archived-circles" => circleCommands.listArchived()

      // Label commands
      case "labels" => interactionCommands.listLabels()
      case "show-label" => interactionCommands.showLabel(args)

      // Interaction commands
      case "log" => interactionCommands.log(args)
      case "remind" | "reminders" => interactionCommands.showReminders()
      case "set-reminder" => interactionCommands.setReminder(args)

      // Other
      case "stats" => interactionCommands.printStats()
      case "save" => ctx.save(); println("Saved.")
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
    list                    List all people
    add <name>              Add a new person
    show-person <name>      Show person details
    edit <name>             Edit a person
    search <query>          Search by name
    archive <name>          Archive a person
    unarchive <name>        Restore archived person
    archived                List archived people

  Circles:
    circles                 List all circles
    add-circle <name>       Create a new circle
    show-circle <name>      Show circle details
    edit-circle <name>      Edit a circle
    archive-circle <name>   Archive a circle
    unarchive-circle <name> Restore archived circle
    archived-circles        List archived circles

  Labels:
    labels                  List all labels
    show-label <name>       Show label details

  Interactions:
    log <name>              Log an interaction
    remind                  Show overdue reminders
    set-reminder <name>     Set reminder frequency

  Other:
    stats                   Show statistics
    save                    Manual save
    help                    Show this help
    exit                    Exit

TIPS:
  - Names are case-insensitive
  - Partial matches work
  - Use quotes for names with spaces: add "Mary Jane"
""".trim)
  }
}
