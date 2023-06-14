/* BRANE AST.h
 *   by Lut99
 *
 * Created:
 *   14 Jun 2023, 11:49:07
 * Last edited:
 *   14 Jun 2023, 18:10:03
 * Auto updated?
 *   Yes
 *
 * Description:
 *   Defines the headers of the `libbrane_ast` library.
**/

#ifndef BRANE_AST_H
#define BRANE_AST_H


/***** ERROR *****/
/* Defines the error type returned by the library.
 * 
 * WARNING: Do not access any internals yourself, since there are no guarantees on the internal layout of this struct.
 */
typedef struct _custom_error Error;

/* Destructor for the Error type.
 *
 * SAFETY: You _must_ call this destructor yourself whenever you are done with the struct to cleanup any code.
 *
 * # Arguments
 * - `err`: The [`Error`] to deallocate.
 */
void error_free(Error* err);

/* Returns if this error contains a message to display or not (and thus whether something went wrong).
 * 
 * # Arguments
 * - `err`: The [`Error`] to check the message status of.
 * 
 * # Returns
 * True if [`error_message()`] can safely be called, or false otherwise.
 */
bool error_occurred(Error* err);

/* Gets the error in this message.
 * 
 * SAFETY: You _must_ check if an error has actually occurred by calling [`error_occurred()`] first.
 * If it returns false, then the return value of this function is undefined.
 * 
 * # Arguments
 * - `err`: The [`Error`] to check the message status of.
 * 
 * # Returns
 * True if [`error_get_message()`] can safely be called, or false otherwise.
 */
char* error_message(Error* err);





/***** FUNCTIONS *****/
/* Returns the BRANE version for which this compiler is valid.
 * 
 * # Returns
 * String version that contains a major, minor and patch version separated by dots.
 */
const char* version();



/* Compiles the given BraneScript snippet to the BRANE Workflow Representation.
 * 
 * Note that the representation is returned as JSON, and not really meant to be inspected from C-code.
 * Use other functions in this library instead to ensure you are compatible with the latest WR version.
 * 
 * # Arguments
 * - `raw`: The raw BraneScript snippet to parse.
 * 
 * # Returns
 * [`NULL`]
 * 
 * # Errors
 * If this function errors, typically because the given snippet is invalid BraneScript, then an [`Error`]-struct is returned instead containing information about what happened.
 */
Error* compile(const char* bs, char** wr);

#endif
