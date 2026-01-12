package network

// ============================================================================
// VALIDATION
// ============================================================================

/**
 * Represents a validation failure with a human-readable message.
 */
case class ValidationError(message: String)

object ValidationError {
  def blankField(fieldName: String): ValidationError = 
    ValidationError(s"$fieldName cannot be blank")
  
  def nonPositive(fieldName: String): ValidationError = 
    ValidationError(s"$fieldName must be positive")
  
  def emptySet(fieldName: String): ValidationError = 
    ValidationError(s"$fieldName cannot be empty")
  
  def notFound(entityType: String, id: String): ValidationError =
    ValidationError(s"$entityType not found: $id")
  
  def alreadyExists(entityType: String, identifier: String): ValidationError =
    ValidationError(s"$entityType already exists: $identifier")
}

/**
 * Validation helpers that return Either[ValidationError, A].
 * Right is success, Left is failure.
 */
object Validation {
  
  /**
   * Validates that a string is not blank (empty or whitespace-only).
   */
  def nonBlank(value: String, fieldName: String): Either[ValidationError, String] =
    if (value.trim.isEmpty) Left(ValidationError.blankField(fieldName))
    else Right(value.trim)
  
  /**
   * Validates that an integer is positive (> 0).
   */
  def positive(value: Int, fieldName: String): Either[ValidationError, Int] =
    if (value <= 0) Left(ValidationError.nonPositive(fieldName))
    else Right(value)
  
  /**
   * Validates that a set is non-empty.
   */
  def nonEmptySet[A](value: Set[A], fieldName: String): Either[ValidationError, Set[A]] =
    if (value.isEmpty) Left(ValidationError.emptySet(fieldName))
    else Right(value)
  
  /**
   * Validates an optional positive integer (None is valid, Some(n) must be positive).
   */
  def optionalPositive(value: Option[Int], fieldName: String): Either[ValidationError, Option[Int]] =
    value match {
      case None => Right(None)
      case Some(n) => positive(n, fieldName).map(Some(_))
    }
}
