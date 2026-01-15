package network

import scala.io.StdIn

/**
 * CLI commands for interactions, reminders, labels, and stats.
 */
class InteractionCommands(ctx: CLIContext) {

  def log(args: List[String]): Unit = {
    ctx.findPerson(args) match {
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
          NetworkOps.logInPersonInteraction(ctx.network, person.id, myLocation, topics, noteOpt)
        } else {
          NetworkOps.logRemoteInteraction(ctx.network, person.id, medium, myLocation, theirLocation, topics, noteOpt)
        }

        ctx.withSave(result) {
          println(s"Logged interaction with ${person.name}")
        }

      case None =>
        if (args.isEmpty) println("Usage: log <n>")
    }
  }

  def showReminders(): Unit = {
    val overdue = NetworkQueries.peopleNeedingReminder(ctx.network)
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
            .map(d => s"last contact ${ctx.formatDaysAgo(d)}")
            .getOrElse("")
          s"$days days overdue ($lastContact)"
      }
      println(s"  ${status.person.name} - $overdueStr")
    }
  }

  def setReminder(args: List[String]): Unit = {
    ctx.findPerson(args) match {
      case Some(person) =>
        print(s"Remind every how many days? (0 to remove reminder): ")
        val input = StdIn.readLine().trim
        scala.util.Try(input.toInt).toOption match {
          case Some(0) =>
            ctx.withSave(NetworkOps.setReminder(ctx.network, person.id, None)) {
              println(s"Reminder removed for ${person.name}")
            }
          case Some(days) if days > 0 =>
            val updated = if (!ctx.network.relationships.contains(person.id)) {
              NetworkOps.setRelationship(ctx.network, person.id, reminderDays = Some(days))
            } else {
              NetworkOps.setReminder(ctx.network, person.id, Some(days))
            }
            ctx.withSave(updated) {
              println(s"Reminder set: reach out to ${person.name} every $days days")
            }
          case _ =>
            println("Invalid number")
        }

      case None =>
        if (args.isEmpty) println("Usage: set-reminder <n>")
    }
  }

  def listLabels(): Unit = {
    val labels = ctx.network.relationshipLabels.values.toList.sortBy(_.name)
    println(s"Relationship labels (${labels.size}):")
    for (label <- labels) {
      val count = NetworkQueries.peopleWithLabel(ctx.network, label.id).size
      println(s"  ${label.name} ($count)")
    }
  }

  def showLabel(args: List[String]): Unit = {
    ctx.findLabel(args) match {
      case Some(label) =>
        println()
        println(s"Label: ${label.name}")

        val people = NetworkQueries.peopleWithLabel(ctx.network, label.id)
        println(s"People: ${if (people.isEmpty) "(none)" else people.map(_.name).mkString(", ")}")
        println()

      case None =>
        if (args.isEmpty) println("Usage: show-label <n>")
    }
  }

  def printStats(): Unit = {
    val s = NetworkQueries.stats(ctx.network)
    println()
    println(s"People: ${s.activePeople} active, ${s.archivedPeople} archived")
    println(s"Interactions: ${s.totalInteractions}")
    println(s"Circles: ${s.activeCircles} active, ${s.archivedCircles} archived")
    if (s.remindersOverdue > 0) {
      println(s"Reminders overdue: ${s.remindersOverdue}")
    }
    println()
  }
}
