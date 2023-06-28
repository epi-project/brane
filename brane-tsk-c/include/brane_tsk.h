/* BRANE TSK.h
 *   by Lut99
 *
 * Created:
 *   14 Jun 2023, 11:49:07
 * Last edited:
 *   28 Jun 2023, 20:10:25
 * Auto updated?
 *   Yes
 *
 * Description:
 *   Defines the headers of the `libbrane_tsk` library.
**/

#ifndef BRANE_TSK_H
#define BRANE_TSK_H


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

/* Returns if this error contains a source warning to display or not.
 * 
 * # Arguments
 * - `err`: The [`Error`] to check the source warning status of.
 * 
 * # Returns
 * True if [`error_print_warns()`] will print anything, or false otherwise.
 */
bool error_warn_occurred(Error* err);
/* Returns if this error contains a source error to display or not (and thus whether something went wrong).
 * 
 * # Arguments
 * - `err`: The [`Error`] to check the source error status of.
 * 
 * # Returns
 * True if [`error_print_errs()`] will print anything, or false otherwise.
 */
bool error_err_occurred(Error* err);
/* Returns if this error contains a message to display or not (and thus whether something went wrong).
 * 
 * # Arguments
 * - `err`: The [`Error`] to check the message status of.
 * 
 * # Returns
 * True if [`error_print_msg()`] will print anything, or false otherwise.
 */
bool error_msg_occurred(Error* err);

/* Prints the warnings in this error to stderr.
 * 
 * The error struct may contain multiple errors if the source code contained those.
 * 
 * # Arguments
 * - `err`: The [`Error`] to print the source warnings of.
 * - `file`: Some string describing the source/filename of the source text.
 * - `source`: The physical source text, as parsed.
 */
void error_print_warns(Error* err, const char* file, const char* source);
/* Prints the errors in this error to stderr.
 * 
 * The error struct may contain multiple errors if the source code contained those.
 * 
 * # Arguments
 * - `err`: The [`Error`] to check the source errors of.
 * - `file`: Some string describing the source/filename of the source text.
 * - `source`: The physical source text, as parsed.
 */
void error_print_errs(Error* err, const char* file, const char* source);
/* Prints the non-source related error to stderr.
 * 
 * This usually indicates a "harder error" that the user did not do with the input source text.
 * 
 * # Arguments
 * - `err`: The [`Error`] to print the message of.
 */
void error_print_msg(Error* err);





/***** COMPILER *****/
/* Defines a BraneScript compiler.
 * 
 * Successive snippets can be compiled with the same compiler to retain state of what is already defined and what not.
 */
typedef struct _compiler Compiler;

/* Constructor for the Compiler.
 * 
 * # Arguments
 * - `endpoint`: The endpoint (as an address) to read the package & data index from. This is the address of a `brane-api` instance.
 * - `compiler`: Will point to the newly created [`Compiler`] when done. **Note**: Has to be manually [`free()`](libc::free())ed.
 * 
 * # Returns
 * An [`Error`]-struct that may or may not contain any generated errors. If [`error_err_occurred()`] is true, though, then `compiler` will point to [`NULL`].
 */
Error* compiler_new(const char* endpoint, Compiler** compiler);
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
 * 
 * # Returns
 * An [`Error`]-struct that may or may not contain any generated errors. If [`error_err_occurred()`] is true, though, then `wr` will point to [`NULL`].
 */
Error* compiler_compile(Compiler* compiler, const char* bs, char** wr);

#endif
