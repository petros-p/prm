package network

import scala.io.StdIn

/**
 * CLI commands for managing circles.
 */
class CircleCommands(ctx: CLIContext) {

  def list(): Unit = {
    val circles = NetworkQueries.activeCircles(ctx.network)
    if (circles.isEmpty) {
      println("No circles yet. Use 'add-circle <n>' to create one.")
    } else {
      println(s"Circles (${circles.size}):")
      for (circle <- circles) {
        val count = circle.memberIds.size
        println(s"  ${circle.name} ($count members)")
      }
    }
  }

  def add(args: List[String]): Unit = {
    val name = if (args.nonEmpty) args.mkString(" ") else {
      print("Circle name: ")
      StdIn.readLine().trim
    }

    if (name.isEmpty) {
      println("Name cannot be empty.")
      return
    }

    println("Add members (enter numbers separated by spaces, or press Enter to skip):")
    val people = NetworkQueries.activePeople(ctx.network)
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

    ctx.withSaveAndResult(NetworkOps.createCircle(ctx.network, name, memberIds = memberIds)) { circle =>
      println(s"Created circle: ${circle.name} with ${memberIds.size} members")
    }
  }

  def show(args: List[String]): Unit = {
    ctx.findCircle(args) match {
      case Some(circle) =>
        println()
        println(s"Name: ${circle.name}")
        println(s"Description: ${circle.description.getOrElse("(none)")}")
        println(s"Archived: ${if (circle.archived) "yes" else "no"}")

        val members = NetworkQueries.circleMembers(ctx.network, circle.id)
        println(s"Members: ${if (members.isEmpty) "(none)" else members.map(_.name).mkString(", ")}")
        println()

      case None =>
        if (args.isEmpty) println("Usage: show-circle <n>")
    }
  }

  def edit(args: List[String]): Unit = {
    ctx.findCircle(args) match {
      case Some(circle) =>
        println(s"Editing circle: ${circle.name}")
        println()

        print(s"Name [${circle.name}]: ")
        val nameInput = StdIn.readLine().trim
        if (nameInput.nonEmpty) {
          ctx.withSave(NetworkOps.updateCircle(ctx.network, circle.id, name = Some(nameInput))) {}
        }

        val currentMembers = NetworkQueries.circleMembers(ctx.network, circle.id).map(_.id).toSet
        val allPeople = NetworkQueries.activePeople(ctx.network)

        println()
        println("Edit members (enter numbers to toggle, press Enter when done):")

        /**
         * Recursively prompts user to toggle circle members until they press Enter.
         * Returns the final set of selected person IDs.
         */
        @scala.annotation.tailrec
        def toggleMembers(selectedIds: Set[Id[Person]]): Set[Id[Person]] = {
          allPeople.zipWithIndex.foreach { case (person, i) =>
            val marker = if (selectedIds.contains(person.id)) "[x]" else "[ ]"
            println(s"  ${i + 1}. $marker ${person.name}")
          }
          print("Toggle (or Enter to finish): ")
          val input = StdIn.readLine().trim

          if (input.isEmpty) {
            selectedIds
          } else {
            val updatedIds = input.split("\\s+").foldLeft(selectedIds) { (ids, s) =>
              scala.util.Try(s.toInt - 1).toOption.flatMap(i => allPeople.lift(i)) match {
                case Some(person) =>
                  if (ids.contains(person.id)) ids - person.id else ids + person.id
                case None =>
                  ids
              }
            }
            toggleMembers(updatedIds)
          }
        }

        val finalSelectedIds = toggleMembers(currentMembers)

        ctx.withSave(NetworkOps.setCircleMembers(ctx.network, circle.id, finalSelectedIds)) {
          println(s"Circle updated with ${finalSelectedIds.size} members")
        }

      case None =>
        if (args.isEmpty) println("Usage: edit-circle <n>")
    }
  }

  def archive(args: List[String]): Unit = {
    ctx.findCircle(args) match {
      case Some(circle) =>
        ctx.withSave(NetworkOps.archiveCircle(ctx.network, circle.id)) {
          println(s"Archived circle: ${circle.name}")
        }
      case None =>
        if (args.isEmpty) println("Usage: archive-circle <n>")
    }
  }

  def unarchive(args: List[String]): Unit = {
    val query = args.mkString(" ")
    if (query.isEmpty) {
      println("Usage: unarchive-circle <n>")
      return
    }

    val archived = NetworkQueries.archivedCircles(ctx.network)
    val matches = archived.filter(_.name.toLowerCase.contains(query.toLowerCase))

    matches match {
      case Nil => println(s"No archived circle found matching '$query'")
      case List(circle) =>
        ctx.withSave(NetworkOps.unarchiveCircle(ctx.network, circle.id)) {
          println(s"Restored circle: ${circle.name}")
        }
      case multiple =>
        println("Multiple matches found:")
        multiple.foreach(c => println(s"  ${c.name}"))
        println("Please be more specific.")
    }
  }

  def listArchived(): Unit = {
    val archived = NetworkQueries.archivedCircles(ctx.network)
    if (archived.isEmpty) {
      println("No archived circles.")
    } else {
      println(s"Archived circles (${archived.size}):")
      archived.foreach(c => println(s"  ${c.name}"))
    }
  }
}
