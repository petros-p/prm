package network

import scala.io.StdIn

/**
 * CLI commands for managing people.
 */
class PersonCommands(ctx: CLIContext) {

  def list(): Unit = {
    val people = NetworkQueries.activePeople(ctx.network)
    if (people.isEmpty) {
      println("No people in your network yet. Use 'add <name>' to add someone.")
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

  def add(args: List[String]): Unit = {
    val name = if (args.nonEmpty) args.mkString(" ") else {
      print("Name: ")
      StdIn.readLine().trim
    }

    if (name.isEmpty) {
      println("Name cannot be empty.")
      return
    }

    ctx.withSaveAndResult(NetworkOps.addPerson(ctx.network, name)) { person =>
      println(s"Added ${person.name}")

      print("How did you meet? (press Enter to skip) ")
      val howWeMet = StdIn.readLine().trim
      if (howWeMet.nonEmpty) {
        ctx.withSave(NetworkOps.updatePerson(ctx.network, person.id, howWeMet = Some(Some(howWeMet)))) {}
      }

      println("Labels (enter numbers separated by spaces, or press Enter to skip):")
      val labels = ctx.network.relationshipLabels.values.toList.sortBy(_.name)
      labels.zipWithIndex.foreach { case (label, i) =>
        println(s"  ${i + 1}. ${label.name}")
      }
      print("Labels: ")
      val labelInput = StdIn.readLine().trim
      if (labelInput.nonEmpty) {
        val indices = labelInput.split("\\s+").flatMap(s => scala.util.Try(s.toInt - 1).toOption)
        val selectedLabels = indices.flatMap(i => labels.lift(i)).map(_.id).toSet
        if (selectedLabels.nonEmpty) {
          ctx.withSave(NetworkOps.setRelationship(ctx.network, person.id, selectedLabels)) {}
        }
      }

      print("Reminder every how many days? (press Enter to skip) ")
      val reminderInput = StdIn.readLine().trim
      if (reminderInput.nonEmpty) {
        scala.util.Try(reminderInput.toInt).toOption match {
          case Some(days) if days > 0 =>
            val rel = ctx.network.relationships.get(person.id)
            if (rel.isDefined) {
              ctx.withSave(NetworkOps.setReminder(ctx.network, person.id, Some(days))) {
                println(s"Reminder set for every $days days")
              }
            } else {
              ctx.withSave(NetworkOps.setRelationship(ctx.network, person.id, reminderDays = Some(days))) {
                println(s"Reminder set for every $days days")
              }
            }
          case _ => // ignore invalid input
        }
      }
    }
  }

  def show(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) =>
        println()
        println(s"Name: ${person.name}")
        println(s"Nickname: ${person.nickname.getOrElse("(none)")}")
        println(s"Birthday: ${person.birthday.map(_.toString).getOrElse("(none)")}")
        println(s"How we met: ${person.howWeMet.getOrElse("(none)")}")
        println(s"Notes: ${person.notes.getOrElse("(none)")}")
        println(s"Default location: ${person.defaultLocation.getOrElse("(none)")}")

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

  def edit(args: List[String]): Unit = {
    ctx.findPerson(args) match {
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
          ctx.network, person.id,
          name = newName,
          nickname = newNickname,
          howWeMet = newHowWeMet,
          notes = newNotes,
          defaultLocation = newLoc
        ) match {
          case Right(n) =>
            ctx.network = n
            ctx.save()
          case Left(e) =>
            println(s"Error: ${e.message}")
            return
        }

        // Labels section
        println()
        val currentLabels = NetworkQueries.labelsFor(ctx.network, person.id)
        val allLabels = ctx.network.relationshipLabels.values.toList.sortBy(_.name)

        println("Labels (enter numbers to toggle, press Enter when done):")
        println(s"Current: ${if (currentLabels.isEmpty) "(none)" else currentLabels.map(_.name).toList.sorted.mkString(", ")}")
        println()

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

        ctx.withSave(NetworkOps.setLabels(ctx.network, person.id, selectedIds)) {
          val finalLabels = selectedIds.flatMap(ctx.network.relationshipLabels.get).map(_.name).toList.sorted
          println()
          println(s"Updated ${newName.getOrElse(person.name)}")
          println(s"Labels: ${if (finalLabels.isEmpty) "(none)" else finalLabels.mkString(", ")}")
        }

      case None =>
        if (args.isEmpty) println("Usage: edit <name>")
    }
  }

  def search(args: List[String]): Unit = {
    if (args.isEmpty) {
      println("Usage: search <query>")
      return
    }

    val query = args.mkString(" ")
    val results = NetworkQueries.findByName(ctx.network, query)

    if (results.isEmpty) {
      println(s"No people found matching '$query'")
    } else {
      println(s"Found ${results.size} match(es):")
      for (person <- results) {
        val archived = if (person.archived) " (archived)" else ""
        val self = if (person.isSelf) " (you)" else ""
        println(s"  ${person.name}$self$archived")
      }
    }
  }

  def archive(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) =>
        ctx.withSave(NetworkOps.archivePerson(ctx.network, person.id)) {
          println(s"Archived ${person.name}")
        }
      case None =>
        if (args.isEmpty) println("Usage: archive <name>")
    }
  }

  def unarchive(args: List[String]): Unit = {
    val query = args.mkString(" ")
    if (query.isEmpty) {
      println("Usage: unarchive <name>")
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
