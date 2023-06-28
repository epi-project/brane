/* BRANE AST.h
 *   by Lut99
 *
 * Created:
 *   14 Jun 2023, 11:49:07
 * Last edited:
 *   15 Jun 2023, 19:31:53
 * Auto updated?
 *   Yes
 *
 * Description:
 *   Defines the headers of the `libbrane_ast` library.
**/

#ifndef BRANE_AST_H
#define BRANE_AST_H


/***** FUNCTIONS *****/
/* Returns the BRANE version for which this compiler is valid.
 * 
 * # Returns
 * String version that contains a major, minor and patch version separated by dots.
 */
const char* version();





/***** ERROR *****/
/* Defines the error type returned by the library.
 * 
 * WARNING: Do not access any internals yourself, since there are no guarantees on the internal layout of this struct.
 */
typedef struct _error Error;

/* Destructor for the Error type.
 *
 * SAFETY: You _must_ call this destructor yourself whenever you are done with the struct to cleanup any code. _Don't_ use any C-library free!
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
 * True if [`error_print_warns()`] will print anything, or false otherwise.
 */
bool error_warn_occurred(Error* err);
/* Returns if this error contains a message to display or not (and thus whether something went wrong).
 * 
 * # Arguments
 * - `err`: The [`Error`] to check the message status of.
 * 
 * # Returns
 * True if [`error_print_errs()`] will print anything, or false otherwise.
 */
bool error_err_occurred(Error* err);

/* Prints the warnings in this error to stderr.
 * 
 * The error struct may contain multiple errors if the source code contained those.
 * 
 * # Arguments
 * - `err`: The [`Error`] to check the message status of.
 */
void error_print_warns(Error* err);
/* Prints the errors in this error to stderr.
 * 
 * The error struct may contain multiple errors if the source code contained those.
 * 
 * # Arguments
 * - `err`: The [`Error`] to check the message status of.
 */
void error_print_errs(Error* err);





/***** COMPILER *****/
/* Defines a BraneScript compiler.
 * 
 * Successive snippets can be compiled with the same compiler to retain state of what is already defined and what not.
 */
typedef struct _compiler Compiler;

/* Constructor for the Compiler.
 * 
 * # Returns
 * A new [`Compiler`] instance.
 */
Compiler* compiler_new();
/* Destructor for the Compiler.
 * 
 * SAFETY: You _must_ call this destructor yourself. _Don't_ use any C-library free!
 * 
 * # Arguments
 * - `compiler`: The [`Compiler`] to free.
 */
void compiler_free(Compiler* compiler);

/* Compiles the given BraneScript snippet to the BRANE Workflow Representation.
 * 
 * Note that the representation is returned as JSON, and not really meant to be inspected from C-code.
 * Use other functions in this library instead to ensure you are compatible with the latest WR version.
 * 
 * # Arguments
 * - `compiler`: The [`Compiler`] to compile with. Essentially this determines which previous compile state to use.
 * - `bs`: The raw BraneScript snippet to parse.
 * - `wr`: Will point to the compiled JSON string when done. **Note**: Has to be manually [`free()`](libc::free())ed.
/// 
/// # Returns
/// An [`Error`]-struct that may or may not contain any generated errors. If [`error_err_occurred()`] is true, though, then `wr` will point to [`NULL`].
 */
Error* compiler_compile(Compiler* compiler, const char* bs, char** wr);

#endif
