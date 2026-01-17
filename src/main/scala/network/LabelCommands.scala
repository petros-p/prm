package network

import scala.io.StdIn

/**
 * CLI commands for managing relationship labels.
 */
class LabelCommands(ctx: CLIContext) {

  /**
   * Lists all active (non-archived) labels with people count.
   */
  def list(): Unit = {
    val labels = NetworkQueries.activeLabels(ctx.network)
    if (labels.isEmpty) {
      println("No labels yet. Use 'add-label <name>' to create one.")
    } else {
      println(s"Labels (${labels.size}):")
      for (label <- labels) {
        val count = NetworkQueries.peopleWithLabel(ctx.network, label.id).size
        println(s"  ${label.name} ($count)")
      }
    }
  }

  /**
   * Shows details for a specific label.
   */
  def show(args: List[String]): Unit = {
    ctx.findLabel(args) match {
      case Some(label) =>
        println()
        println(s"Label: ${label.name}")
        println(s"Archived: ${if (label.archived) "yes" else "no"}")

        val people = NetworkQueries.peopleWithLabel(ctx.network, label.id)
        println(s"People: ${if (people.isEmpty) "(none)" else people.map(_.name).mkString(", ")}")
        println()

      case None =>
        if (args.isEmpty) println("Usage: show-label <name>")
    }
  }

  /**
   * Adds a new label.
   */
  def add(args: List[String]): Unit = {
    val name = if (args.nonEmpty) args.mkString(" ") else {
      print("Label name: ")
      StdIn.readLine().trim
    }

    if (name.isEmpty) {
      println("Name cannot be empty.")
      return
    }

    ctx.withSaveAndResult(NetworkOps.addLabel(ctx.network, name)) { label =>
      println(s"Created label: ${label.name}")
    }
  }

  /**
   * Edit menu for a label - can edit name and/or assigned people.
   */
  def edit(args: List[String]): Unit = {
    ctx.findLabel(args) match {
      case Some(label) =>
        println(s"Editing label: ${label.name}")
        println()
        println("What would you like to edit?")
        println("  1. Name")
        println("  2. People with this label")
        println("  3. Both")
        println()
        print("Choice (1-3, or Enter to cancel): ")
        
        val choice = StdIn.readLine().trim
        choice match {
          case "1" => editName(label)
          case "2" => editPeople(label)
          case "3" => 
            editName(label)
            // Refresh label after name change
            ctx.network.relationshipLabels.get(label.id).foreach(editPeople)
          case "" => println("Cancelled.")
          case _ => println("Invalid choice.")
        }

      case None =>
        if (args.isEmpty) println("Usage: edit-label <name>")
    }
  }

  /**
   * Edit just the label name.
   */
  private def editName(label: RelationshipLabel): Unit = {
    print(s"Name [${label.name}]: ")
    val nameInput = StdIn.readLine().trim
    if (nameInput.nonEmpty && nameInput != label.name) {
      ctx.withSave(NetworkOps.updateLabel(ctx.network, label.id, name = Some(nameInput))) {
        println(s"Renamed label to: $nameInput")
      }
    }
  }

  /**
   * Edit which people have this label using toggle interface.
   */
  private def editPeople(label: RelationshipLabel): Unit = {
    val allPeople = NetworkQueries.activePeople(ctx.network).filter(!_.isSelf)
    if (allPeople.isEmpty) {
      println("No people to assign labels to.")
      return
    }

    val currentPeople = NetworkQueries.peopleWithLabel(ctx.network, label.id).map(_.id).toSet
    
    println()
    println("Toggle people with this label (enter numbers, Enter when done):")

    @scala.annotation.tailrec
    def togglePeople(selectedIds: Set[Id[Person]]): Set[Id[Person]] = {
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
            case None => ids
          }
        }
        togglePeople(updatedIds)
      }
    }

    val finalSelectedIds = togglePeople(currentPeople)
    
    // Update relationships: add label to newly selected, remove from unselected
    val toAdd = finalSelectedIds -- currentPeople
    val toRemove = currentPeople -- finalSelectedIds

    toAdd.foreach { personId =>
      NetworkOps.addLabels(ctx.network, personId, Set(label.id)) match {
        case Right(n) => ctx.network = n
        case Left(e) => println(s"Error: ${e.message}")
      }
    }

    toRemove.foreach { personId =>
      NetworkOps.removeLabels(ctx.network, personId, Set(label.id)) match {
        case Right(n) => ctx.network = n
        case Left(e) => println(s"Error: ${e.message}")
      }
    }

    ctx.save()
    println(s"Updated: ${finalSelectedIds.size} people now have label '${label.name}'")
  }

  /**
   * Archives a label (hides from selection lists, preserves associations).
   */
  def archive(args: List[String]): Unit = {
    ctx.findLabel(args) match {
      case Some(label) =>
        ctx.withSave(NetworkOps.archiveLabel(ctx.network, label.id)) {
          println(s"Archived label: ${label.name}")
        }
      case None =>
        if (args.isEmpty) println("Usage: archive-label <name>")
    }
  }

  /**
   * Unarchives a label.
   */
  def unarchive(args: List[String]): Unit = {
    val query = args.mkString(" ")
    if (query.isEmpty) {
      println("Usage: unarchive-label <name>")
      return
    }

    val archived = NetworkQueries.archivedLabels(ctx.network)
    val matches = archived.filter(_.name.toLowerCase.contains(query.toLowerCase))

    matches match {
      case Nil => println(s"No archived label found matching '$query'")
      case List(label) =>
        ctx.withSave(NetworkOps.unarchiveLabel(ctx.network, label.id)) {
          println(s"Restored label: ${label.name}")
        }
      case multiple =>
        println("Multiple matches found:")
        multiple.foreach(l => println(s"  ${l.name}"))
        println("Please be more specific.")
    }
  }

  /**
   * Lists all archived labels.
   */
  def listArchived(): Unit = {
    val archived = NetworkQueries.archivedLabels(ctx.network)
    if (archived.isEmpty) {
      println("No archived labels.")
    } else {
      println(s"Archived labels (${archived.size}):")
      archived.foreach(l => println(s"  ${l.name}"))
    }
  }
}
