package network

import upickle.default.*
import java.time.LocalDate
import java.util.UUID

// ============================================================================
// JSON SERIALIZATION
// ============================================================================

/**
 * JSON serialization for the network model using upickle.
 * 
 * Usage:
 *   import network.JsonCodecs.given
 *   
 *   // Serialize to JSON string
 *   val json = write(network)
 *   
 *   // Deserialize from JSON string
 *   val network = read[Network](json)
 *   
 *   // Pretty-printed JSON
 *   val prettyJson = write(network, indent = 2)
 */
object JsonCodecs {

  // --------------------------------------------------------------------------
  // PRIMITIVE TYPE CODECS
  // --------------------------------------------------------------------------

  /**
   * UUID is serialized as a string.
   */
  given ReadWriter[UUID] = readwriter[String].bimap[UUID](
    uuid => uuid.toString,
    str => UUID.fromString(str)
  )

  /**
   * LocalDate is serialized as ISO-8601 string (YYYY-MM-DD).
   */
  given ReadWriter[LocalDate] = readwriter[String].bimap[LocalDate](
    date => date.toString,
    str => LocalDate.parse(str)
  )

  /**
   * Generic Id[A] codec - serializes the inner UUID.
   */
  given [A]: ReadWriter[Id[A]] = readwriter[UUID].bimap[Id[A]](
    id => id.value,
    uuid => Id(uuid)
  )

  // --------------------------------------------------------------------------
  // USER CODEC
  // --------------------------------------------------------------------------

  given ReadWriter[User] = macroRW[User]

  // --------------------------------------------------------------------------
  // CONTACT INFO CODECS
  // --------------------------------------------------------------------------

  given ReadWriter[Address] = macroRW[Address]
  given ReadWriter[CustomContactType] = macroRW[CustomContactType]

  // For sealed traits, we need to handle each case
  given ReadWriter[ContactType] = readwriter[ujson.Value].bimap[ContactType](
    {
      case ContactType.Phone => ujson.Obj("type" -> "Phone")
      case ContactType.Email => ujson.Obj("type" -> "Email")
      case ContactType.PhysicalAddress => ujson.Obj("type" -> "PhysicalAddress")
      case ContactType.Custom(typeId) => ujson.Obj("type" -> "Custom", "typeId" -> typeId.value.toString)
    },
    json => {
      val obj = json.obj
      obj("type").str match {
        case "Phone" => ContactType.Phone
        case "Email" => ContactType.Email
        case "PhysicalAddress" => ContactType.PhysicalAddress
        case "Custom" => ContactType.Custom(Id(java.util.UUID.fromString(obj("typeId").str)))
      }
    }
  )

  given ReadWriter[ContactValue] = readwriter[ujson.Value].bimap[ContactValue](
    {
      case ContactValue.StringValue(v) => ujson.Obj("type" -> "String", "value" -> v)
      case ContactValue.AddressValue(v) => ujson.Obj(
        "type" -> "Address",
        "street" -> v.street,
        "city" -> v.city,
        "state" -> v.state,
        "zip" -> v.zip,
        "country" -> v.country
      )
    },
    json => {
      val obj = json.obj
      obj("type").str match {
        case "String" => ContactValue.StringValue(obj("value").str)
        case "Address" => ContactValue.AddressValue(Address(
          street = obj("street").str,
          city = obj("city").str,
          state = obj("state").str,
          zip = obj("zip").str,
          country = obj("country").str
        ))
      }
    }
  )

  given ReadWriter[ContactEntry] = macroRW[ContactEntry]

  // --------------------------------------------------------------------------
  // INTERACTION CODECS
  // --------------------------------------------------------------------------

  given ReadWriter[InteractionMedium] = readwriter[String].bimap[InteractionMedium](
    {
      case InteractionMedium.InPerson => "InPerson"
      case InteractionMedium.Text => "Text"
      case InteractionMedium.PhoneCall => "PhoneCall"
      case InteractionMedium.VideoCall => "VideoCall"
      case InteractionMedium.SocialMedia => "SocialMedia"
    },
    {
      case "InPerson" => InteractionMedium.InPerson
      case "Text" => InteractionMedium.Text
      case "PhoneCall" => InteractionMedium.PhoneCall
      case "VideoCall" => InteractionMedium.VideoCall
      case "SocialMedia" => InteractionMedium.SocialMedia
    }
  )

  given ReadWriter[Interaction] = macroRW[Interaction]

  // --------------------------------------------------------------------------
  // MODEL TYPE CODECS
  // --------------------------------------------------------------------------

  /**
   * Person codec.
   */
  given ReadWriter[Person] = macroRW[Person]

  /**
   * RelationshipLabel codec.
   */
  given ReadWriter[RelationshipLabel] = macroRW[RelationshipLabel]

  /**
   * Relationship codec.
   */
  given ReadWriter[Relationship] = macroRW[Relationship]

  /**
   * Circle codec.
   */
  given ReadWriter[Circle] = macroRW[Circle]

  /**
   * Network codec - the main entry point for serialization.
   */
  given ReadWriter[Network] = macroRW[Network]

  // --------------------------------------------------------------------------
  // CONVENIENCE FUNCTIONS
  // --------------------------------------------------------------------------

  /**
   * Serializes a network to a JSON string.
   */
  def toJson(network: Network, pretty: Boolean = false): String =
    if (pretty) write(network, indent = 2)
    else write(network)

  /**
   * Deserializes a network from a JSON string.
   * Returns Left(error message) on failure, Right(network) on success.
   */
  def fromJson(json: String): Either[String, Network] =
    try Right(read[Network](json))
    catch case e: Exception => Left(e.getMessage)

  /**
   * Serializes a network to a JSON file.
   */
  def saveToFile(network: Network, path: String, pretty: Boolean = true): Either[String, Unit] =
    try {
      val json = toJson(network, pretty)
      val writer = new java.io.PrintWriter(path)
      try {
        writer.write(json)
        Right(())
      } finally {
        writer.close()
      }
    } catch case e: Exception => Left(e.getMessage)

  /**
   * Loads a network from a JSON file.
   */
  def loadFromFile(path: String): Either[String, Network] =
    try {
      val source = scala.io.Source.fromFile(path)
      try {
        val json = source.mkString
        fromJson(json)
      } finally {
        source.close()
      }
    } catch case e: Exception => Left(e.getMessage)
}
