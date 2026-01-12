package network

import java.time.LocalDate
import java.time.temporal.ChronoUnit

// ============================================================================
// NETWORK OPERATIONS
// ============================================================================

/**
 * Pure helper functions for common network operations.
 * All functions return Either[ValidationError, Network] for operations that can fail,
 * or the result directly for operations that cannot fail.
 */
object NetworkOps {

  // --------------------------------------------------------------------------
  // USER OPERATIONS (stubs for future multi-user support)
  // --------------------------------------------------------------------------

  /**
   * Creates a new user.
   * Note: This does not handle authentication. That will be added later.
   */
  def createUser(name: String, email: String): Either[ValidationError, User] =
    for {
      validName <- Validation.nonBlank(name, "name")
      validEmail <- Validation.nonBlank(email, "email")
    } yield User.create(validName, validEmail)

  /**
   * Creates a new network for a user.
   */
  def createNetwork(owner: User, selfName: String): Either[ValidationError, Network] =
    for {
      validName <- Validation.nonBlank(selfName, "selfName")
    } yield {
      val self = Person.createSelf(validName)
      Network.create(owner, self)
    }

  // --------------------------------------------------------------------------
  // PERSON OPERATIONS
  // --------------------------------------------------------------------------

  /**
   * Adds a new person to the network.
   * Does NOT create a relationship - that's a separate step.
   */
  def addPerson(
    network: Network,
    name: String,
    nickname: Option[String] = None,
    howWeMet: Option[String] = None,
    birthday: Option[LocalDate] = None,
    notes: Option[String] = None,
    defaultLocation: Option[String] = None,
    contactInfo: List[ContactEntry] = List.empty
  ): Either[ValidationError, (Network, Person)] =
    for {
      validName <- Validation.nonBlank(name, "name")
    } yield {
      val person = Person.create(
        name = validName,
        nickname = nickname.map(_.trim).filter(_.nonEmpty),
        howWeMet = howWeMet.map(_.trim).filter(_.nonEmpty),
        birthday = birthday,
        notes = notes.map(_.trim).filter(_.nonEmpty),
        defaultLocation = defaultLocation.map(_.trim).filter(_.nonEmpty),
        contactInfo = contactInfo
      )
      val updatedNetwork = network.copy(
        people = network.people + (person.id -> person)
      )
      (updatedNetwork, person)
    }

  /**
   * Updates an existing person's information.
   */
  def updatePerson(
    network: Network,
    personId: Id[Person],
    name: Option[String] = None,
    nickname: Option[Option[String]] = None,
    howWeMet: Option[Option[String]] = None,
    birthday: Option[Option[LocalDate]] = None,
    notes: Option[Option[String]] = None,
    defaultLocation: Option[Option[String]] = None
  ): Either[ValidationError, Network] =
    network.people.get(personId) match {
      case None => Left(ValidationError.notFound("Person", personId.value.toString))
      case Some(person) =>
        val nameValidation = name match {
          case Some(n) => Validation.nonBlank(n, "name").map(Some(_))
          case None => Right(None)
        }
        
        nameValidation.map { validName =>
          val updated = person.copy(
            name = validName.getOrElse(person.name),
            nickname = nickname.getOrElse(person.nickname),
            howWeMet = howWeMet.getOrElse(person.howWeMet),
            birthday = birthday.getOrElse(person.birthday),
            notes = notes.getOrElse(person.notes),
            defaultLocation = defaultLocation.getOrElse(person.defaultLocation)
          )
          network.copy(people = network.people.updated(personId, updated))
        }
    }

  /**
   * Archives a person. Their data is preserved but they're hidden from main views.
   */
  def archivePerson(network: Network, personId: Id[Person]): Either[ValidationError, Network] =
    network.people.get(personId) match {
      case None => Left(ValidationError.notFound("Person", personId.value.toString))
      case Some(person) if person.isSelf => Left(ValidationError("Cannot archive self"))
      case Some(person) =>
        val archived = person.copy(archived = true)
        Right(network.copy(people = network.people.updated(personId, archived)))
    }

  /**
   * Unarchives a person, making them visible in main views again.
   */
  def unarchivePerson(network: Network, personId: Id[Person]): Either[ValidationError, Network] =
    network.people.get(personId) match {
      case None => Left(ValidationError.notFound("Person", personId.value.toString))
      case Some(person) =>
        val unarchived = person.copy(archived = false)
        Right(network.copy(people = network.people.updated(personId, unarchived)))
    }

  // --------------------------------------------------------------------------
  // CONTACT INFO OPERATIONS
  // --------------------------------------------------------------------------

  /**
   * Creates a new custom contact type (e.g., "Discord", "LinkedIn").
   */
  def createCustomContactType(
    network: Network,
    name: String
  ): Either[ValidationError, (Network, CustomContactType)] =
    for {
      validName <- Validation.nonBlank(name, "name")
      _ <- {
        val exists = network.customContactTypes.values.exists(_.name.equalsIgnoreCase(validName))
        if (exists) Left(ValidationError.alreadyExists("CustomContactType", validName))
        else Right(())
      }
    } yield {
      val contactType = CustomContactType.create(validName)
      val updatedNetwork = network.copy(
        customContactTypes = network.customContactTypes + (contactType.id -> contactType)
      )
      (updatedNetwork, contactType)
    }

  /**
   * Adds a phone number to a person's contact info.
   */
  def addPhone(
    network: Network,
    personId: Id[Person],
    number: String,
    label: Option[String] = None
  ): Either[ValidationError, Network] =
    for {
      validNumber <- Validation.nonBlank(number, "phone number")
      result <- addContactEntry(network, personId, ContactEntry.phone(validNumber, label.map(_.trim).filter(_.nonEmpty)))
    } yield result

  /**
   * Adds an email address to a person's contact info.
   */
  def addEmail(
    network: Network,
    personId: Id[Person],
    email: String,
    label: Option[String] = None
  ): Either[ValidationError, Network] =
    for {
      validEmail <- Validation.nonBlank(email, "email")
      result <- addContactEntry(network, personId, ContactEntry.email(validEmail, label.map(_.trim).filter(_.nonEmpty)))
    } yield result

  /**
   * Adds a physical address to a person's contact info.
   */
  def addAddress(
    network: Network,
    personId: Id[Person],
    street: String,
    city: String,
    state: String,
    zip: String,
    country: String,
    label: Option[String] = None
  ): Either[ValidationError, Network] =
    for {
      validStreet <- Validation.nonBlank(street, "street")
      validCity <- Validation.nonBlank(city, "city")
      validState <- Validation.nonBlank(state, "state")
      validZip <- Validation.nonBlank(zip, "zip")
      validCountry <- Validation.nonBlank(country, "country")
      address = Address(validStreet, validCity, validState, validZip, validCountry)
      result <- addContactEntry(network, personId, ContactEntry.address(address, label.map(_.trim).filter(_.nonEmpty)))
    } yield result

  /**
   * Adds a custom contact entry to a person's contact info.
   */
  def addCustomContact(
    network: Network,
    personId: Id[Person],
    typeId: Id[CustomContactType],
    value: String,
    label: Option[String] = None
  ): Either[ValidationError, Network] =
    if (!network.customContactTypes.contains(typeId))
      Left(ValidationError.notFound("CustomContactType", typeId.value.toString))
    else
      for {
        validValue <- Validation.nonBlank(value, "value")
        result <- addContactEntry(network, personId, ContactEntry.custom(typeId, validValue, label.map(_.trim).filter(_.nonEmpty)))
      } yield result

  /**
   * Internal helper to add a contact entry to a person.
   */
  private def addContactEntry(
    network: Network,
    personId: Id[Person],
    entry: ContactEntry
  ): Either[ValidationError, Network] =
    network.people.get(personId) match {
      case None => Left(ValidationError.notFound("Person", personId.value.toString))
      case Some(person) =>
        val updated = person.copy(contactInfo = person.contactInfo :+ entry)
        Right(network.copy(people = network.people.updated(personId, updated)))
    }

  /**
   * Removes a contact entry from a person.
   */
  def removeContactEntry(
    network: Network,
    personId: Id[Person],
    entryId: Id[ContactEntry]
  ): Either[ValidationError, Network] =
    network.people.get(personId) match {
      case None => Left(ValidationError.notFound("Person", personId.value.toString))
      case Some(person) =>
        val updated = person.copy(contactInfo = person.contactInfo.filterNot(_.id == entryId))
        Right(network.copy(people = network.people.updated(personId, updated)))
    }

  /**
   * Updates the label on a contact entry.
   */
  def updateContactEntryLabel(
    network: Network,
    personId: Id[Person],
    entryId: Id[ContactEntry],
    newLabel: Option[String]
  ): Either[ValidationError, Network] =
    network.people.get(personId) match {
      case None => Left(ValidationError.notFound("Person", personId.value.toString))
      case Some(person) =>
        val updatedContacts = person.contactInfo.map { entry =>
          if (entry.id == entryId) entry.copy(label = newLabel.map(_.trim).filter(_.nonEmpty))
          else entry
        }
        val updated = person.copy(contactInfo = updatedContacts)
        Right(network.copy(people = network.people.updated(personId, updated)))
    }

  // --------------------------------------------------------------------------
  // RELATIONSHIP OPERATIONS
  // --------------------------------------------------------------------------

  /**
   * Creates a relationship with a person (or updates if one exists).
   */
  def setRelationship(
    network: Network,
    personId: Id[Person],
    labels: Set[Id[RelationshipLabel]] = Set.empty,
    reminderDays: Option[Int] = None
  ): Either[ValidationError, Network] =
    if (!network.people.contains(personId))
      Left(ValidationError.notFound("Person", personId.value.toString))
    else if (personId == network.selfId)
      Left(ValidationError("Cannot create relationship with self"))
    else
      Validation.optionalPositive(reminderDays, "reminderDays").map { validReminder =>
        val existing = network.relationships.get(personId)
        val relationship = existing match {
          case Some(r) => r.copy(labels = labels, reminderDays = validReminder)
          case None => Relationship.create(personId, labels, validReminder)
        }
        network.copy(relationships = network.relationships.updated(personId, relationship))
      }

  /**
   * Sets the labels for a person's relationship, replacing any existing labels.
   */
  def setLabels(
    network: Network,
    personId: Id[Person],
    labels: Set[Id[RelationshipLabel]]
  ): Either[ValidationError, Network] =
    network.relationships.get(personId) match {
      case None => 
        // Create a new relationship with just these labels
        if (!network.people.contains(personId))
          Left(ValidationError.notFound("Person", personId.value.toString))
        else if (personId == network.selfId)
          Left(ValidationError("Cannot create relationship with self"))
        else {
          val relationship = Relationship.create(personId, labels)
          Right(network.copy(relationships = network.relationships.updated(personId, relationship)))
        }
      case Some(rel) =>
        val updated = rel.copy(labels = labels)
        Right(network.copy(relationships = network.relationships.updated(personId, updated)))
    }

  /**
   * Adds labels to an existing relationship.
   */
  def addLabels(
    network: Network,
    personId: Id[Person],
    labels: Set[Id[RelationshipLabel]]
  ): Either[ValidationError, Network] =
    network.relationships.get(personId) match {
      case None => Left(ValidationError.notFound("Relationship", personId.value.toString))
      case Some(rel) =>
        val updated = rel.copy(labels = rel.labels ++ labels)
        Right(network.copy(relationships = network.relationships.updated(personId, updated)))
    }

  /**
   * Removes labels from an existing relationship.
   */
  def removeLabels(
    network: Network,
    personId: Id[Person],
    labels: Set[Id[RelationshipLabel]]
  ): Either[ValidationError, Network] =
    network.relationships.get(personId) match {
      case None => Left(ValidationError.notFound("Relationship", personId.value.toString))
      case Some(rel) =>
        val updated = rel.copy(labels = rel.labels -- labels)
        Right(network.copy(relationships = network.relationships.updated(personId, updated)))
    }

  /**
   * Sets the reminder frequency for a relationship.
   */
  def setReminder(
    network: Network,
    personId: Id[Person],
    days: Option[Int]
  ): Either[ValidationError, Network] =
    network.relationships.get(personId) match {
      case None => Left(ValidationError.notFound("Relationship", personId.value.toString))
      case Some(rel) =>
        Validation.optionalPositive(days, "days").map { validDays =>
          val updated = rel.copy(reminderDays = validDays)
          network.copy(relationships = network.relationships.updated(personId, updated))
        }
    }

  // --------------------------------------------------------------------------
  // INTERACTION OPERATIONS
  // --------------------------------------------------------------------------

  /**
   * Logs an in-person interaction with a person.
   * Creates a relationship if one doesn't exist.
   */
  def logInPersonInteraction(
    network: Network,
    personId: Id[Person],
    location: String,
    topics: Set[String],
    note: Option[String] = None,
    date: LocalDate = LocalDate.now()
  ): Either[ValidationError, Network] =
    if (!network.people.contains(personId))
      Left(ValidationError.notFound("Person", personId.value.toString))
    else if (personId == network.selfId)
      Left(ValidationError("Cannot log interaction with self"))
    else
      for {
        validLocation <- Validation.nonBlank(location, "location")
        validTopics <- Validation.nonEmptySet(topics.map(_.trim).filter(_.nonEmpty), "topics")
      } yield {
        val interaction = Interaction(
          id = Id.generate[Interaction],
          date = date,
          medium = InteractionMedium.InPerson,
          myLocation = validLocation,
          theirLocation = Some(validLocation),
          topics = validTopics,
          note = note.map(_.trim).filter(_.nonEmpty)
        )
        
        val relationship = network.relationships.getOrElse(
          personId,
          Relationship.create(personId)
        )
        val updated = relationship.copy(
          interactionHistory = interaction :: relationship.interactionHistory
        )
        network.copy(relationships = network.relationships.updated(personId, updated))
      }

  /**
   * Logs a remote interaction (Text, Phone Call, Video Call, Social Media) with a person.
   * Creates a relationship if one doesn't exist.
   */
  def logRemoteInteraction(
    network: Network,
    personId: Id[Person],
    medium: InteractionMedium,
    myLocation: String,
    theirLocation: Option[String],
    topics: Set[String],
    note: Option[String] = None,
    date: LocalDate = LocalDate.now()
  ): Either[ValidationError, Network] =
    if (!network.people.contains(personId))
      Left(ValidationError.notFound("Person", personId.value.toString))
    else if (personId == network.selfId)
      Left(ValidationError("Cannot log interaction with self"))
    else if (medium == InteractionMedium.InPerson)
      Left(ValidationError("Use logInPersonInteraction for in-person interactions"))
    else
      for {
        validMyLocation <- Validation.nonBlank(myLocation, "myLocation")
        validTopics <- Validation.nonEmptySet(topics.map(_.trim).filter(_.nonEmpty), "topics")
      } yield {
        val interaction = Interaction(
          id = Id.generate[Interaction],
          date = date,
          medium = medium,
          myLocation = validMyLocation,
          theirLocation = theirLocation.map(_.trim).filter(_.nonEmpty),
          topics = validTopics,
          note = note.map(_.trim).filter(_.nonEmpty)
        )
        
        val relationship = network.relationships.getOrElse(
          personId,
          Relationship.create(personId)
        )
        val updated = relationship.copy(
          interactionHistory = interaction :: relationship.interactionHistory
        )
        network.copy(relationships = network.relationships.updated(personId, updated))
      }

  /**
   * Legacy method for backwards compatibility.
   * Logs an in-person interaction.
   */
  def logInteraction(
    network: Network,
    personId: Id[Person],
    location: String,
    topics: Set[String],
    note: Option[String] = None,
    date: LocalDate = LocalDate.now()
  ): Either[ValidationError, Network] =
    logInPersonInteraction(network, personId, location, topics, note, date)

  // --------------------------------------------------------------------------
  // CIRCLE OPERATIONS
  // --------------------------------------------------------------------------

  /**
   * Creates a new circle.
   */
  def createCircle(
    network: Network,
    name: String,
    description: Option[String] = None,
    memberIds: Set[Id[Person]] = Set.empty
  ): Either[ValidationError, (Network, Circle)] =
    for {
      validName <- Validation.nonBlank(name, "name")
    } yield {
      val circle = Circle.create(
        name = validName,
        description = description.map(_.trim).filter(_.nonEmpty),
        memberIds = memberIds.filter(network.people.contains)
      )
      val updatedNetwork = network.copy(
        circles = network.circles + (circle.id -> circle)
      )
      (updatedNetwork, circle)
    }

  /**
   * Updates a circle's name and/or description.
   */
  def updateCircle(
    network: Network,
    circleId: Id[Circle],
    name: Option[String] = None,
    description: Option[Option[String]] = None
  ): Either[ValidationError, Network] =
    network.circles.get(circleId) match {
      case None => Left(ValidationError.notFound("Circle", circleId.value.toString))
      case Some(circle) =>
        val nameValidation = name match {
          case Some(n) => Validation.nonBlank(n, "name").map(Some(_))
          case None => Right(None)
        }
        
        nameValidation.map { validName =>
          val updated = circle.copy(
            name = validName.getOrElse(circle.name),
            description = description.getOrElse(circle.description)
          )
          network.copy(circles = network.circles.updated(circleId, updated))
        }
    }

  /**
   * Adds members to a circle.
   */
  def addToCircle(
    network: Network,
    circleId: Id[Circle],
    personIds: Set[Id[Person]]
  ): Either[ValidationError, Network] =
    network.circles.get(circleId) match {
      case None => Left(ValidationError.notFound("Circle", circleId.value.toString))
      case Some(circle) =>
        val validIds = personIds.filter(network.people.contains)
        val updated = circle.copy(memberIds = circle.memberIds ++ validIds)
        Right(network.copy(circles = network.circles.updated(circleId, updated)))
    }

  /**
   * Removes members from a circle.
   */
  def removeFromCircle(
    network: Network,
    circleId: Id[Circle],
    personIds: Set[Id[Person]]
  ): Either[ValidationError, Network] =
    network.circles.get(circleId) match {
      case None => Left(ValidationError.notFound("Circle", circleId.value.toString))
      case Some(circle) =>
        val updated = circle.copy(memberIds = circle.memberIds -- personIds)
        Right(network.copy(circles = network.circles.updated(circleId, updated)))
    }

  /**
   * Sets the members of a circle, replacing any existing members.
   */
  def setCircleMembers(
    network: Network,
    circleId: Id[Circle],
    memberIds: Set[Id[Person]]
  ): Either[ValidationError, Network] =
    network.circles.get(circleId) match {
      case None => Left(ValidationError.notFound("Circle", circleId.value.toString))
      case Some(circle) =>
        val validIds = memberIds.filter(network.people.contains)
        val updated = circle.copy(memberIds = validIds)
        Right(network.copy(circles = network.circles.updated(circleId, updated)))
    }

  /**
   * Archives a circle. Data is preserved but hidden from main views.
   */
  def archiveCircle(network: Network, circleId: Id[Circle]): Either[ValidationError, Network] =
    network.circles.get(circleId) match {
      case None => Left(ValidationError.notFound("Circle", circleId.value.toString))
      case Some(circle) =>
        val archived = circle.copy(archived = true)
        Right(network.copy(circles = network.circles.updated(circleId, archived)))
    }

  /**
   * Unarchives a circle, making it visible in main views again.
   */
  def unarchiveCircle(network: Network, circleId: Id[Circle]): Either[ValidationError, Network] =
    network.circles.get(circleId) match {
      case None => Left(ValidationError.notFound("Circle", circleId.value.toString))
      case Some(circle) =>
        val unarchived = circle.copy(archived = false)
        Right(network.copy(circles = network.circles.updated(circleId, unarchived)))
    }

  /**
   * Deletes a circle (does not affect the people in it).
   */
  def deleteCircle(network: Network, circleId: Id[Circle]): Either[ValidationError, Network] =
    if (!network.circles.contains(circleId))
      Left(ValidationError.notFound("Circle", circleId.value.toString))
    else
      Right(network.copy(circles = network.circles - circleId))

  // --------------------------------------------------------------------------
  // LABEL OPERATIONS
  // --------------------------------------------------------------------------

  /**
   * Adds a new relationship label.
   */
  def addLabel(network: Network, name: String): Either[ValidationError, (Network, RelationshipLabel)] =
    for {
      validName <- Validation.nonBlank(name, "name")
      _ <- {
        val exists = network.relationshipLabels.values.exists(_.name.equalsIgnoreCase(validName))
        if (exists) Left(ValidationError.alreadyExists("Label", validName))
        else Right(())
      }
    } yield {
      val label = RelationshipLabel.create(validName)
      val updatedNetwork = network.copy(
        relationshipLabels = network.relationshipLabels + (label.id -> label)
      )
      (updatedNetwork, label)
    }
}
