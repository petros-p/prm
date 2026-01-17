package network

import scala.io.StdIn
import java.time.LocalDate
import java.time.format.DateTimeFormatter

/**
 * CLI commands for managing people.
 */
class PersonCommands(ctx: CLIContext) {

  private val dateFormatter = DateTimeFormatter.ofPattern("yyyy-MM-dd")

  /**
   * Lists all active (non-archived) people.
   */
  def list(): Unit = {
    val people = NetworkQueries.activePeople(ctx.network)
    if (people.isEmpty) {
      println("No people in your network yet. Use 'add-person' to add someone.")
      return
    }

    println(s"People in your network (${people.size}):")
    println()
    for (person <- people) {
      val labels = NetworkQueries.labelsFor(ctx.network, person.id).map(_.name).toList.sorted
      val labelStr = if (labels.nonEmpty) s" [${labels.mkString(", ")}]" else ""
      val lastContact = NetworkQueries.daysSinceInteraction(ctx.network, person.id)
        .map(d => s" - last contact: ${ctx.formatDaysAgo(d)}")
        .getOrElse("")
      println(s"  ${person.name}$labelStr$lastContact")
    }
  }

  /**
   * Adds a new person with full interactive flow.
   * User can press 's' at any prompt to save and exit.
   * All fields except name are optional (press Enter to skip).
   */
  def add(args: List[String]): Unit = {
    println("Adding a new person (press Enter to skip optional fields, 's' to save and exit)")
    println()

    // Name (mandatory)
    val name = if (args.nonEmpty) args.mkString(" ") else {
      print("Name (required): ")
      val input = StdIn.readLine().trim
      if (input.toLowerCase == "s" || input.isEmpty) {
        if (input.isEmpty) println("Name is required.")
        return
      }
      input
    }

    if (name.isEmpty) {
      println("Name cannot be empty.")
      return
    }

    // Create person first
    val addResult = NetworkOps.addPerson(ctx.network, name)
    val person = addResult match {
      case Right((n, p)) =>
        ctx.network = n
        ctx.save()
        println(s"Added ${p.name}")
        p
      case Left(e) =>
        println(s"Error: ${e.message}")
        return
    }

    // Helper to check for save-and-exit
    def checkSaveExit(input: String): Boolean = input.toLowerCase == "s"

    // Nickname
    print("Nickname: ")
    val nicknameInput = StdIn.readLine().trim
    if (checkSaveExit(nicknameInput)) { println("Saved."); return }
    if (nicknameInput.nonEmpty) {
      updateField(person.id, nickname = Some(Some(nicknameInput)))
    }

    // Birthday
    print("Birthday (YYYY-MM-DD): ")
    val birthdayInput = StdIn.readLine().trim
    if (checkSaveExit(birthdayInput)) { println("Saved."); return }
    if (birthdayInput.nonEmpty) {
      parseDate(birthdayInput) match {
        case Some(date) => updateField(person.id, birthday = Some(Some(date)))
        case None => println("Invalid date format, skipping.")
      }
    }

    // How we met
    print("How did you meet: ")
    val howWeMetInput = StdIn.readLine().trim
    if (checkSaveExit(howWeMetInput)) { println("Saved."); return }
    if (howWeMetInput.nonEmpty) {
      updateField(person.id, howWeMet = Some(Some(howWeMetInput)))
    }

    // Notes
    print("Notes: ")
    val notesInput = StdIn.readLine().trim
    if (checkSaveExit(notesInput)) { println("Saved."); return }
    if (notesInput.nonEmpty) {
      updateField(person.id, notes = Some(Some(notesInput)))
    }

    // Location
    print("Location: ")
    val locationInput = StdIn.readLine().trim
    if (checkSaveExit(locationInput)) { println("Saved."); return }
    if (locationInput.nonEmpty) {
      updateField(person.id, location = Some(Some(locationInput)))
    }

    // Labels
    println()
    print("Add labels? (y/n): ")
    val labelsChoice = StdIn.readLine().trim.toLowerCase
    if (labelsChoice == "s") { println("Saved."); return }
    if (labelsChoice == "y") {
      selectLabels(person.id)
    }

    // Circles
    println()
    print("Add to circles? (y/n): ")
    val circlesChoice = StdIn.readLine().trim.toLowerCase
    if (circlesChoice == "s") { println("Saved."); return }
    if (circlesChoice == "y") {
      selectCircles(person.id)
    }

    // Phones
    println()
    print("Add phone numbers? (y/n): ")
    val phonesChoice = StdIn.readLine().trim.toLowerCase
    if (phonesChoice == "s") { println("Saved."); return }
    if (phonesChoice == "y") {
      addPhones(person.id)
    }

    // Emails
    println()
    print("Add email addresses? (y/n): ")
    val emailsChoice = StdIn.readLine().trim.toLowerCase
    if (emailsChoice == "s") { println("Saved."); return }
    if (emailsChoice == "y") {
      addEmails(person.id)
    }

    // Reminder
    println()
    print("Set reminder? (y/n): ")
    val reminderChoice = StdIn.readLine().trim.toLowerCase
    if (reminderChoice == "s") { println("Saved."); return }
    if (reminderChoice == "y") {
      setReminderFor(person.id)
    }

    // Log interaction
    println()
    print("Log an interaction now? (y/n): ")
    val interactionChoice = StdIn.readLine().trim.toLowerCase
    if (interactionChoice == "s") { println("Saved."); return }
    if (interactionChoice == "y") {
      ctx.network.people.get(person.id).foreach { p =>
        new InteractionCommands(ctx).logForPerson(p)
      }
    }

    println()
    println(s"Finished adding ${name}.")
  }

  /**
   * Shows details for a specific person.
   */
  def show(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) =>
        println()
        println(s"Name: ${person.name}")
        println(s"Nickname: ${person.nickname.getOrElse("(none)")}")
        println(s"Birthday: ${person.birthday.map(_.toString).getOrElse("(none)")}")
        println(s"How we met: ${person.howWeMet.getOrElse("(none)")}")
        println(s"Notes: ${person.notes.getOrElse("(none)")}")
        println(s"Location: ${person.location.getOrElse("(none)")}")

        val labels = NetworkQueries.labelsFor(ctx.network, person.id)
        println(s"Labels: ${if (labels.isEmpty) "(none)" else labels.map(_.name).toList.sorted.mkString(", ")}")

        val circles = NetworkQueries.circlesFor(ctx.network, person.id)
        println(s"Circles: ${if (circles.isEmpty) "(none)" else circles.map(_.name).mkString(", ")}")

        val contacts = NetworkQueries.allContactsFor(ctx.network, person.id)
        println(s"Phones: ${formatContacts(contacts, ContactType.Phone)}")
        println(s"Emails: ${formatContacts(contacts, ContactType.Email)}")

        val reminder = ctx.network.relationships.get(person.id).flatMap(_.reminderDays)
        println(s"Reminder: ${reminder.map(d => s"every $d days").getOrElse("(none)")}")

        NetworkQueries.lastInteractionWith(ctx.network, person.id) match {
          case Some(interaction) =>
            val daysAgo = NetworkQueries.daysSinceInteraction(ctx.network, person.id).getOrElse(0L)
            val mediumStr = InteractionMedium.name(interaction.medium)
            println(s"Last interaction: ${ctx.formatDaysAgo(daysAgo)} via $mediumStr")
          case None =>
            println("Last interaction: (never)")
        }

        val interactions = NetworkQueries.interactionsWith(ctx.network, person.id)
        println(s"Total interactions: ${interactions.size}")
        println()

      case None =>
        if (args.isEmpty) println("Usage: show-person <name>")
    }
  }

  /**
   * Edit menu for a person - shows all editable fields.
   */
  def edit(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) =>
        println(s"Editing ${person.name}")
        println()
        println("What would you like to edit?")
        println("  1. Name")
        println("  2. Nickname")
        println("  3. Birthday")
        println("  4. How we met")
        println("  5. Notes")
        println("  6. Location")
        println("  7. Labels")
        println("  8. Circles")
        println("  9. Phone numbers")
        println(" 10. Email addresses")
        println()
        print("Choice (1-10, or Enter to cancel): ")
        
        val choice = StdIn.readLine().trim
        choice match {
          case "1" => editNameCmd(person)
          case "2" => editNicknameCmd(person)
          case "3" => editBirthdayCmd(person)
          case "4" => editHowWeMetCmd(person)
          case "5" => editNotesCmd(person)
          case "6" => editLocationCmd(person)
          case "7" => editLabelsCmd(person)
          case "8" => editCirclesCmd(person)
          case "9" => editPhonesCmd(person)
          case "10" => editEmailsCmd(person)
          case "" => println("Cancelled.")
          case _ => println("Invalid choice.")
        }

      case None =>
        if (args.isEmpty) println("Usage: edit-person <name>")
    }
  }

  // ============================================================================
  // GRANULAR EDIT COMMANDS
  // ============================================================================

  /**
   * Edits a person's name via direct command.
   */
  def editName(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) => editNameCmd(person)
      case None => if (args.isEmpty) println("Usage: edit-name <name>")
    }
  }

  /**
   * Edits a person's nickname via direct command.
   */
  def editNickname(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) => editNicknameCmd(person)
      case None => if (args.isEmpty) println("Usage: edit-nickname <name>")
    }
  }

  /**
   * Edits a person's birthday via direct command.
   */
  def editBirthday(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) => editBirthdayCmd(person)
      case None => if (args.isEmpty) println("Usage: edit-birthday <name>")
    }
  }

  /**
   * Edits how you met a person via direct command.
   */
  def editHowWeMet(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) => editHowWeMetCmd(person)
      case None => if (args.isEmpty) println("Usage: edit-how-we-met <name>")
    }
  }

  /**
   * Edits a person's notes via direct command.
   */
  def editNotes(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) => editNotesCmd(person)
      case None => if (args.isEmpty) println("Usage: edit-notes <name>")
    }
  }

  /**
   * Edits a person's location via direct command.
   */
  def editLocation(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) => editLocationCmd(person)
      case None => if (args.isEmpty) println("Usage: edit-location <name>")
    }
  }

  /**
   * Edits a person's labels via direct command.
   */
  def editLabels(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) => editLabelsCmd(person)
      case None => if (args.isEmpty) println("Usage: edit-labels <name>")
    }
  }

  /**
   * Edits a person's circle memberships via direct command.
   */
  def editCircles(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) => editCirclesCmd(person)
      case None => if (args.isEmpty) println("Usage: edit-circles <name>")
    }
  }

  /**
   * Edits a person's phone numbers via direct command.
   */
  def editPhone(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) => editPhonesCmd(person)
      case None => if (args.isEmpty) println("Usage: edit-phone <name>")
    }
  }

  /**
   * Edits a person's email addresses via direct command.
   */
  def editEmail(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) => editEmailsCmd(person)
      case None => if (args.isEmpty) println("Usage: edit-email <name>")
    }
  }

  // ============================================================================
  // INTERNAL EDIT IMPLEMENTATIONS
  // ============================================================================

  /**
   * Prompts for and updates a person's name.
   */
  private def editNameCmd(person: Person): Unit = {
    print(s"Name [${person.name}]: ")
    val input = StdIn.readLine().trim
    if (input.nonEmpty) {
      ctx.withSave(NetworkOps.updatePerson(ctx.network, person.id, name = Some(input))) {
        println(s"Updated name to: $input")
      }
    }
  }

  /**
   * Prompts for and updates a person's nickname.
   */
  private def editNicknameCmd(person: Person): Unit = {
    print(s"Nickname [${person.nickname.getOrElse("")}] (enter 'clear' to remove): ")
    val input = StdIn.readLine().trim
    if (input.nonEmpty) {
      val newValue = if (input.toLowerCase == "clear") None else Some(input)
      ctx.withSave(NetworkOps.updatePerson(ctx.network, person.id, nickname = Some(newValue))) {
        newValue match {
          case Some(v) => println(s"Updated nickname to: $v")
          case None => println("Cleared nickname.")
        }
      }
    }
  }

  /**
   * Prompts for and updates a person's birthday.
   */
  private def editBirthdayCmd(person: Person): Unit = {
    print(s"Birthday [${person.birthday.map(_.toString).getOrElse("")}] (YYYY-MM-DD, 'clear' to remove): ")
    val input = StdIn.readLine().trim
    if (input.nonEmpty) {
      if (input.toLowerCase == "clear") {
        ctx.withSave(NetworkOps.updatePerson(ctx.network, person.id, birthday = Some(None))) {
          println("Cleared birthday.")
        }
      } else {
        parseDate(input) match {
          case Some(date) =>
            ctx.withSave(NetworkOps.updatePerson(ctx.network, person.id, birthday = Some(Some(date)))) {
              println(s"Updated birthday to: $date")
            }
          case None => println("Invalid date format.")
        }
      }
    }
  }

  /**
   * Prompts for and updates how you met a person.
   */
  private def editHowWeMetCmd(person: Person): Unit = {
    print(s"How we met [${person.howWeMet.getOrElse("")}] ('clear' to remove): ")
    val input = StdIn.readLine().trim
    if (input.nonEmpty) {
      val newValue = if (input.toLowerCase == "clear") None else Some(input)
      ctx.withSave(NetworkOps.updatePerson(ctx.network, person.id, howWeMet = Some(newValue))) {
        newValue match {
          case Some(v) => println(s"Updated: $v")
          case None => println("Cleared.")
        }
      }
    }
  }

  /**
   * Prompts for and updates a person's notes.
   */
  private def editNotesCmd(person: Person): Unit = {
    print(s"Notes [${person.notes.getOrElse("")}] ('clear' to remove): ")
    val input = StdIn.readLine().trim
    if (input.nonEmpty) {
      val newValue = if (input.toLowerCase == "clear") None else Some(input)
      ctx.withSave(NetworkOps.updatePerson(ctx.network, person.id, notes = Some(newValue))) {
        newValue match {
          case Some(v) => println(s"Updated notes.")
          case None => println("Cleared notes.")
        }
      }
    }
  }

  /**
   * Prompts for and updates a person's location.
   */
  private def editLocationCmd(person: Person): Unit = {
    print(s"Location [${person.location.getOrElse("")}] ('clear' to remove): ")
    val input = StdIn.readLine().trim
    if (input.nonEmpty) {
      val newValue = if (input.toLowerCase == "clear") None else Some(input)
      ctx.withSave(NetworkOps.updatePerson(ctx.network, person.id, location = Some(newValue))) {
        newValue match {
          case Some(v) => println(s"Updated location to: $v")
          case None => println("Cleared location.")
        }
      }
    }
  }

  /**
   * Delegates to selectLabels for label editing.
   */
  private def editLabelsCmd(person: Person): Unit = {
    selectLabels(person.id)
  }

  /**
   * Delegates to selectCircles for circle editing.
   */
  private def editCirclesCmd(person: Person): Unit = {
    selectCircles(person.id)
  }

  /**
   * Interactive editor for a person's phone numbers.
   * Allows adding and removing phone entries.
   */
  private def editPhonesCmd(person: Person): Unit = {
    val phones = person.contactInfo.filter(_.contactType == ContactType.Phone)
    println()
    println("Current phones:")
    if (phones.isEmpty) {
      println("  (none)")
    } else {
      phones.zipWithIndex.foreach { case (entry, i) =>
        val label = entry.label.map(l => s" ($l)").getOrElse("")
        val value = entry.value match {
          case ContactValue.StringValue(v) => v
          case _ => ""
        }
        println(s"  ${i + 1}. $value$label")
      }
    }
    println()
    println("Options: 'add' to add, number to remove, Enter to finish")
    
    @scala.annotation.tailrec
    def editLoop(): Unit = {
      print("Action: ")
      val input = StdIn.readLine().trim.toLowerCase
      input match {
        case "" => // done
        case "add" =>
          print("Phone number: ")
          val number = StdIn.readLine().trim
          if (number.nonEmpty) {
            print("Label (optional): ")
            val label = StdIn.readLine().trim
            val labelOpt = if (label.isEmpty) None else Some(label)
            ctx.withSave(NetworkOps.addPhone(ctx.network, person.id, number, labelOpt)) {
              println(s"Added: $number")
            }
          }
          editLoop()
        case n if n.forall(_.isDigit) =>
          val idx = n.toInt - 1
          val currentPhones = ctx.network.people.get(person.id).map(_.contactInfo.filter(_.contactType == ContactType.Phone)).getOrElse(List.empty)
          currentPhones.lift(idx) match {
            case Some(entry) =>
              ctx.withSave(NetworkOps.removeContactEntry(ctx.network, person.id, entry.id)) {
                println("Removed.")
              }
            case None => println("Invalid number.")
          }
          editLoop()
        case _ =>
          println("Invalid input.")
          editLoop()
      }
    }
    editLoop()
  }

  /**
   * Interactive editor for a person's email addresses.
   * Allows adding and removing email entries.
   */
  private def editEmailsCmd(person: Person): Unit = {
    val emails = person.contactInfo.filter(_.contactType == ContactType.Email)
    println()
    println("Current emails:")
    if (emails.isEmpty) {
      println("  (none)")
    } else {
      emails.zipWithIndex.foreach { case (entry, i) =>
        val label = entry.label.map(l => s" ($l)").getOrElse("")
        val value = entry.value match {
          case ContactValue.StringValue(v) => v
          case _ => ""
        }
        println(s"  ${i + 1}. $value$label")
      }
    }
    println()
    println("Options: 'add' to add, number to remove, Enter to finish")
    
    @scala.annotation.tailrec
    def editLoop(): Unit = {
      print("Action: ")
      val input = StdIn.readLine().trim.toLowerCase
      input match {
        case "" => // done
        case "add" =>
          print("Email address: ")
          val email = StdIn.readLine().trim
          if (email.nonEmpty) {
            print("Label (optional): ")
            val label = StdIn.readLine().trim
            val labelOpt = if (label.isEmpty) None else Some(label)
            ctx.withSave(NetworkOps.addEmail(ctx.network, person.id, email, labelOpt)) {
              println(s"Added: $email")
            }
          }
          editLoop()
        case n if n.forall(_.isDigit) =>
          val idx = n.toInt - 1
          val currentEmails = ctx.network.people.get(person.id).map(_.contactInfo.filter(_.contactType == ContactType.Email)).getOrElse(List.empty)
          currentEmails.lift(idx) match {
            case Some(entry) =>
              ctx.withSave(NetworkOps.removeContactEntry(ctx.network, person.id, entry.id)) {
                println("Removed.")
              }
            case None => println("Invalid number.")
          }
          editLoop()
        case _ =>
          println("Invalid input.")
          editLoop()
      }
    }
    editLoop()
  }

  // ============================================================================
  // HELPER METHODS
  // ============================================================================

  /**
   * Updates one or more fields on a person and saves.
   * Wraps NetworkOps.updatePerson with automatic save.
   */
  private def updateField(
    personId: Id[Person],
    name: Option[String] = None,
    nickname: Option[Option[String]] = None,
    howWeMet: Option[Option[String]] = None,
    birthday: Option[Option[LocalDate]] = None,
    notes: Option[Option[String]] = None,
    location: Option[Option[String]] = None
  ): Unit = {
    NetworkOps.updatePerson(ctx.network, personId, name, nickname, howWeMet, birthday, notes, location) match {
      case Right(n) =>
        ctx.network = n
        ctx.save()
      case Left(e) =>
        println(s"Error: ${e.message}")
    }
  }

  /**
   * Parses a date string in YYYY-MM-DD format.
   * Returns None if the format is invalid.
   */
  private def parseDate(input: String): Option[LocalDate] = {
    try Some(LocalDate.parse(input, dateFormatter))
    catch case _: Exception => None
  }

  /**
   * Interactive toggle interface for selecting labels for a person.
   * Displays all active labels with checkboxes, user toggles by number.
   */
  private def selectLabels(personId: Id[Person]): Unit = {
    val allLabels = NetworkQueries.activeLabels(ctx.network)
    if (allLabels.isEmpty) {
      println("No labels available.")
      return
    }

    val currentLabels = NetworkQueries.labelsFor(ctx.network, personId).map(_.id)

    println("Select labels (enter numbers to toggle, Enter when done):")

    @scala.annotation.tailrec
    def toggleLabels(selectedIds: Set[Id[RelationshipLabel]]): Set[Id[RelationshipLabel]] = {
      allLabels.zipWithIndex.foreach { case (label, i) =>
        val marker = if (selectedIds.contains(label.id)) "[x]" else "[ ]"
        println(s"  ${i + 1}. $marker ${label.name}")
      }
      print("Toggle (or Enter to finish): ")
      val input = StdIn.readLine().trim

      if (input.isEmpty) {
        selectedIds
      } else {
        val updatedIds = input.split("\\s+").foldLeft(selectedIds) { (ids, s) =>
          scala.util.Try(s.toInt - 1).toOption.flatMap(i => allLabels.lift(i)) match {
            case Some(label) =>
              if (ids.contains(label.id)) ids - label.id else ids + label.id
            case None => ids
          }
        }
        toggleLabels(updatedIds)
      }
    }

    val finalSelectedIds = toggleLabels(currentLabels)
    ctx.withSave(NetworkOps.setLabels(ctx.network, personId, finalSelectedIds)) {
      val names = finalSelectedIds.flatMap(ctx.network.relationshipLabels.get).map(_.name).toList.sorted
      println(s"Labels: ${if (names.isEmpty) "(none)" else names.mkString(", ")}")
    }
  }

  /**
   * Interactive toggle interface for selecting circles for a person.
   * Displays all active circles with checkboxes, user toggles by number.
   */
  private def selectCircles(personId: Id[Person]): Unit = {
    val allCircles = NetworkQueries.activeCircles(ctx.network)
    if (allCircles.isEmpty) {
      println("No circles available.")
      return
    }

    val currentCircles = NetworkQueries.circlesFor(ctx.network, personId).map(_.id).toSet

    println("Select circles (enter numbers to toggle, Enter when done):")

    @scala.annotation.tailrec
    def toggleCircles(selectedIds: Set[Id[Circle]]): Set[Id[Circle]] = {
      allCircles.zipWithIndex.foreach { case (circle, i) =>
        val marker = if (selectedIds.contains(circle.id)) "[x]" else "[ ]"
        println(s"  ${i + 1}. $marker ${circle.name}")
      }
      print("Toggle (or Enter to finish): ")
      val input = StdIn.readLine().trim

      if (input.isEmpty) {
        selectedIds
      } else {
        val updatedIds = input.split("\\s+").foldLeft(selectedIds) { (ids, s) =>
          scala.util.Try(s.toInt - 1).toOption.flatMap(i => allCircles.lift(i)) match {
            case Some(circle) =>
              if (ids.contains(circle.id)) ids - circle.id else ids + circle.id
            case None => ids
          }
        }
        toggleCircles(updatedIds)
      }
    }

    val finalSelectedIds = toggleCircles(currentCircles)

    // Update circles: add to newly selected, remove from unselected
    val toAdd = finalSelectedIds -- currentCircles
    val toRemove = currentCircles -- finalSelectedIds

    toAdd.foreach { circleId =>
      NetworkOps.addToCircle(ctx.network, circleId, Set(personId)) match {
        case Right(n) => ctx.network = n
        case Left(e) => println(s"Error: ${e.message}")
      }
    }

    toRemove.foreach { circleId =>
      NetworkOps.removeFromCircle(ctx.network, circleId, Set(personId)) match {
        case Right(n) => ctx.network = n
        case Left(e) => println(s"Error: ${e.message}")
      }
    }

    ctx.save()
    val names = finalSelectedIds.flatMap(ctx.network.circles.get).map(_.name).toList.sorted
    println(s"Circles: ${if (names.isEmpty) "(none)" else names.mkString(", ")}")
  }

  /**
   * Prompts to add multiple phone numbers in a loop.
   * Continues until user presses Enter without input.
   */
  private def addPhones(personId: Id[Person]): Unit = {
    @scala.annotation.tailrec
    def addLoop(): Unit = {
      print("Phone number (or Enter to finish): ")
      val number = StdIn.readLine().trim
      if (number.nonEmpty) {
        print("Label (optional): ")
        val label = StdIn.readLine().trim
        val labelOpt = if (label.isEmpty) None else Some(label)
        ctx.withSave(NetworkOps.addPhone(ctx.network, personId, number, labelOpt)) {
          println(s"Added: $number")
        }
        addLoop()
      }
    }
    addLoop()
  }

  /**
   * Prompts to add multiple email addresses in a loop.
   * Continues until user presses Enter without input.
   */
  private def addEmails(personId: Id[Person]): Unit = {
    @scala.annotation.tailrec
    def addLoop(): Unit = {
      print("Email address (or Enter to finish): ")
      val email = StdIn.readLine().trim
      if (email.nonEmpty) {
        print("Label (optional): ")
        val label = StdIn.readLine().trim
        val labelOpt = if (label.isEmpty) None else Some(label)
        ctx.withSave(NetworkOps.addEmail(ctx.network, personId, email, labelOpt)) {
          println(s"Added: $email")
        }
        addLoop()
      }
    }
    addLoop()
  }

  /**
   * Prompts for and sets a reminder frequency for a person.
   * Creates a relationship if one doesn't exist.
   */
  private def setReminderFor(personId: Id[Person]): Unit = {
    print("Remind every how many days: ")
    val input = StdIn.readLine().trim
    scala.util.Try(input.toInt).toOption match {
      case Some(days) if days > 0 =>
        val updated = if (!ctx.network.relationships.contains(personId)) {
          NetworkOps.setRelationship(ctx.network, personId, reminderDays = Some(days))
        } else {
          NetworkOps.setReminder(ctx.network, personId, Some(days))
        }
        ctx.withSave(updated) {
          println(s"Reminder set for every $days days")
        }
      case _ =>
        println("Invalid number, skipping reminder.")
    }
  }

  /**
   * Formats contact entries of a given type as a comma-separated string.
   * Returns "(none)" if no entries of that type exist.
   */
  private def formatContacts(contacts: List[ContactEntry], contactType: ContactType): String = {
    val filtered = contacts.filter(_.contactType == contactType)
    if (filtered.isEmpty) "(none)"
    else filtered.map { entry =>
      val label = entry.label.map(l => s" ($l)").getOrElse("")
      val value = entry.value match {
        case ContactValue.StringValue(v) => v
        case ContactValue.AddressValue(a) => s"${a.street}, ${a.city}"
      }
      s"$value$label"
    }.mkString(", ")
  }

  // ============================================================================
  // FIND COMMAND (searches people, circles, labels)
  // ============================================================================

  /**
   * Searches across people, circles, and labels by name.
   * Displays all matches grouped by type.
   */
  def find(args: List[String]): Unit = {
    if (args.isEmpty) {
      println("Usage: find <query>")
      return
    }

    val query = args.mkString(" ")
    
    // Search people
    val people = NetworkQueries.findByName(ctx.network, query)
    
    // Search circles
    val circles = ctx.network.circles.values
      .filter(_.name.toLowerCase.contains(query.toLowerCase))
      .toList.sortBy(_.name)
    
    // Search labels
    val labels = NetworkQueries.findLabelByName(ctx.network, query)

    val totalResults = people.size + circles.size + labels.size

    if (totalResults == 0) {
      println(s"No results found for '$query'")
    } else {
      println(s"Found $totalResults result(s) for '$query':")
      println()

      if (people.nonEmpty) {
        println("People:")
        for (person <- people) {
          val archived = if (person.archived) " (archived)" else ""
          val self = if (person.isSelf) " (you)" else ""
          println(s"  ${person.name}$self$archived")
        }
        println()
      }

      if (circles.nonEmpty) {
        println("Circles:")
        for (circle <- circles) {
          val archived = if (circle.archived) " (archived)" else ""
          println(s"  ${circle.name}$archived")
        }
        println()
      }

      if (labels.nonEmpty) {
        println("Labels:")
        for (label <- labels) {
          val archived = if (label.archived) " (archived)" else ""
          println(s"  ${label.name}$archived")
        }
      }
    }
  }

  // ============================================================================
  // ARCHIVE COMMANDS
  // ============================================================================

  /**
   * Archives a person, hiding them from main views.
   */
  def archive(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) =>
        ctx.withSave(NetworkOps.archivePerson(ctx.network, person.id)) {
          println(s"Archived ${person.name}")
        }
      case None =>
        if (args.isEmpty) println("Usage: archive-person <name>")
    }
  }

  /**
   * Restores an archived person to active status.
   */
  def unarchive(args: List[String]): Unit = {
    val query = args.mkString(" ")
    if (query.isEmpty) {
      println("Usage: unarchive-person <name>")
      return
    }

    val archived = NetworkQueries.archivedPeople(ctx.network)
    val matches = archived.filter(p =>
      p.name.toLowerCase.contains(query.toLowerCase) ||
      p.nickname.exists(_.toLowerCase.contains(query.toLowerCase))
    )

    matches match {
      case Nil => println(s"No archived person found matching '$query'")
      case List(person) =>
        ctx.withSave(NetworkOps.unarchivePerson(ctx.network, person.id)) {
          println(s"Restored ${person.name}")
        }
      case multiple =>
        println("Multiple matches found:")
        multiple.foreach(p => println(s"  ${p.name}"))
        println("Please be more specific.")
    }
  }

  /**
   * Lists all archived people.
   */
  def listArchived(): Unit = {
    val archived = NetworkQueries.archivedPeople(ctx.network)
    if (archived.isEmpty) {
      println("No archived people.")
    } else {
      println(s"Archived people (${archived.size}):")
      archived.foreach(p => println(s"  ${p.name}"))
    }
  }
}
