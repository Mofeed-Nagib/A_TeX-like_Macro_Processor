
# A TeX-like Macro Processor

## Assignment Description

TeX (or its later evolution LaTeX) is a program used often in academia to write technical papers and documents. Users define macros in text files that also contain the contents of the document, and TeX processes macros and string expands them based on their definitions.

For this assignment, you will implement a TeX-like macro processor in Rust. This macro processor will perform a transform on a set of input files <strong>(or the standard input when no files are specified)</strong> and output the result to the standard output. As the input is read, your program will replace macro strings according to the macro’s value mapping and the macro expansion rules. Any input file(s) are provided as command line arguments. The execution of your program on the command line should have this form:

<pre style="text-align: center;">cargo run -r [file]*</pre>

Macros always start with an unescaped backslash followed by a name string. Macros have optional arguments. Each argument is placed in curly braces immediately after the macro name. For example:

<pre style="text-align: center;">\NAME{ARGUMENT 1}{ARGUMENT 2}{ARGUMENT 3}</pre>

Here's a brief example of an input/output execution:

<table style="border-collapse: collapse; width: 100%; height: 177px;" border="1">
  <tbody>
    <tr>
      <td style="width: 50%;"><strong>Input</strong></td>
      <td style="width: 50%;"><strong>Output</strong></td>
    </tr>
    <tr style="height: 177px;">
      <td style="width: 50%; height: 177px;">
        <pre>A list of values:<br>\def{MACRO}{VALUE = #}<br>\MACRO{1}<br>\MACRO{2}<br>\MACRO{3}<br>\MACRO{4}<br>\MACRO{5}<br>\MACRO{6}<br>\MACRO{7}</pre>
      </td>
      <td style="width: 50%; height: 177px;">
        <pre>A list of values:<br><br>VALUE = 1<br>VALUE = 2<br>VALUE = 3<br>VALUE = 4<br>VALUE = 5<br>VALUE = 6<br>VALUE = 7</pre>
      </td>
    </tr>
  </tbody>
</table>

The <code>\def</code> macro defines a new macro called <code>\MACRO</code>, Future occurrences of <code>\MACRO</code> will be replaced with <code>VALUE = #</code> where the <code>#</code> is replaced with the argument to <code>\MACRO</code>. We will go into more detail on <code>\def</code> in the section below.

Some notes about the general macro grammar:

<ul>
  <li>Macro names must only contain a string of letters, or numbers.</li>
  <li>No white space is allowed between a macro name and the arguments or between the arguments.</li>
  <li>With a few exceptions, macro arguments can contain arbitrary text, including macro expressions or fragments of macro expressions. The exceptions are for built-in macros, see the section below.</li>
  <li>Macro arguments must be brace balanced (i.e., the number of unescaped left braces is greater than or equal to the number of unescaped right braces in every prefix and equal in the entire string). For example:
    <pre>\valid{this arg {{ is }} balanced}</pre>
  </li>
  <li>With a few exceptions (see the built-in macros below), macro arguments can contain escape characters (backslashes). The details on how escape characters should be handled are given in the section below.</li>
</ul>

<p>&nbsp;</p>

### Recursion and Evaluation Strategy

Your program should read the input from the first character to the end of file (EOF) for each file and process/expand macros as it reads. After expanding a macro, your processor should continue processing at the beginning of the replacement string or value. You should NOT try to eagerly/recursively expand (except for the <code>\expandafter</code> case, see below in the "Built-in Macros" section). Expansions resume once you have done the replacement. This is because the macro's replacement value could be either whole macros or even fragments of macro text, take a look at the following example:

<table style="border-collapse: collapse; width: 100%; height: 177px;" border="1">
  <tbody>
  <tr>
    <td style="width: 50%;"><strong>Input</strong></td>
    <td style="width: 50%;"><strong>Output</strong></td>
  </tr>
  <tr style="height: 177px;">
    <td style="width: 50%; height: 177px;">
      <pre>\def{testMacro}{some text #}<br>\def{macroFragment}{goes #}<br><br>\testMacro{\macroFragment}{here}</pre>
    </td>
    <td style="width: 50%; height: 177px;">
      <pre><br><br><br>some text goes here</pre>
    </td>
  </tr>
  </tbody>
</table>

First we define two macros:

<ul>
  <li><code>\testMacro</code> maps to <code>some text #</code></li>
  <li><code>\macroFragment </code>maps to <code>goes #</code></li>
</ul>

All user-defined macros have only one argument, so the <code>\testMacro</code> is expanded with one argument <code>{\macroFragment}</code> to:

<pre style="text-align: center;">some text \macroFragment{here}</pre>

<p>Then the <code>\macroFragment{here}</code> is expanded to: <code>goes here</code>. Now we're ready to talk about how all of the built-in macros need to be implemented.</p>

<p>&nbsp;</p>

### Built-In Macros

You will need to implement a set of built-in macros in the macro processor you are building. &nbsp;The programmer can use these special macros in the source files to define/undefine new macros, include text from other files, do comparisons, etc. These are listed below:

<ul>
  <li><code><strong>\def</strong><strong></strong></code> allows a programmer to define a new macro mapping:
  <pre>\def{NAME}{VALUE}</pre>
  The entire <code>\def</code> macro and arguments are always replaced by the empty string. &nbsp;The argument <code>NAME</code> must be a nonempty alphanumeric string (can be arbitrarily long). &nbsp;As usual, the <code>VALUE</code> argument must be brace balanced, but can contain arbitrary text. After processing a <code>\def</code> macro and its arguments, &nbsp;the <code>NAME</code> argument is now mapped to the <code>VALUE</code> argument. In the future, macros with that name are valid: the <code>\NAME{ARG}</code> macro should be replaced by <code>VALUE</code>—with each occurrence of the unescaped character <code>#</code> replaced by the argument string (<code>ARG</code>). <strong><em>Note: custom-defined macros must always have exactly one argument, if the <code>VALUE</code> doesn’t have any unescaped <code>#</code> characters, the argument is ignored.</em></strong><br><br></li>
  <li><code><strong>\undef</strong></code> undefines previously defined macro. <code>\undef</code> is replaced by the empty string:
    <pre>\undef{NAME}</pre>
  </li>
  
  <li><strong><code>\if</code></strong> allows text to be processed conditionally (like an if-then-else block). Your implementation should consider false as the empty string and true for any non-empty string. The <code>\if</code> macro should have the following form:
  <pre>\if{COND}{THEN}{ELSE}</pre>
  Like all macros, all three arguments can contain arbitrary text, but must be braced balanced. &nbsp;You should <strong>not</strong> expand <code>COND</code>. The functionality should be this: the entire <code>\if</code> macro including arguments should be replaced with either the <code>THEN</code> or <code>ELSE</code> depending on the size of <code>COND</code>. Then, after expansion, processing resumes at the beginning of the replacement string.<br><br>    </li>
  
  <li><strong><code>\ifdef</code></strong> is similar to <code>\if</code>; it expands to either <code>THEN</code> or <code>ELSE</code>:
  <pre>\ifdef{NAME}{THEN}{ELSE}</pre>
  The main difference is with the condition argument, <code>NAME</code>, which is restricted to alphanumeric characters. If <code>NAME</code> matches a currently defined macro name then the condition is true, otherwise, it is false.<br><br></li>
  
  <li><strong><code>\include</code></strong> macros are replaced by the contents of the file <code>PATH</code>. Typical brace balancing rules apply here:
  <pre>\include{PATH}</pre>
  </li>
  
  <li><code><strong>\expandafter</strong></code> has the form:
  <pre>\expandafter{BEFORE}{AFTER}</pre>
  The point of this macro is to delay expanding the before argument until the after argument has been expanded. The output of this macro expansion is simply <code>BEFORE</code> immediately followed by the expanded <code>AFTER</code>. Note that this changes the recursive evaluation rule, i.e. you should eagerly expand all macros in the <code>AFTER</code> string before touching <code>BEFORE</code>. This means that any new macros defined in <code>AFTER</code> should be in scope for the <code>BEFORE</code>. You may not use additional processes/threads to accomplish these actions. Here’s an example program:<br><br>
  <table style="border-collapse: collapse; width: 100%; height: 177px;" border="1">
    <tbody>
      <tr>
        <td style="width: 60.3256%;"><strong>Input</strong></td>
        <td style="width: 39.6744%;"><strong>Output</strong></td>
      </tr>
      <tr style="height: 177px;">
        <td style="width: 60.3256%; height: 177px;">
          <pre>\def{B}{buffalo}<br>\expandafter{\B{}}{\undef{B}\def{B}{bison}}</pre>
        </td>
        <td style="width: 39.6744%; height: 177px;">
          <pre><br>bison</pre>
        </td>
      </tr>
    </tbody>
  </table>
  Why is this the case? It is because <code>\B{}</code> is expanded after it has been redefined in the <code>AFTER </code>argument. Here are the steps to process these macros:<br>
    <ol type="1">
      <li><code>AFTER</code> should be fully expanded by running your expansion algorithm recursively (including the removal of certain escape characters in normal text, see the section below).</li>
      <li>the result of the above expansion should be appended to the (unexpanded) <code>BEFORE</code> argument</li>
      <li>the <code>\expandafter</code> macro and arguments are now replaced with the above concatenation.</li>
      <li>standard expansion processing should continue, starting from the start of <code>BEFORE</code>.</li>
    </ol>
  </li>
</ul>

<p>&nbsp;</p>

### Comments

Your program should support comments. The comment character, <code>%</code>, should cause your program to ignore it and all subsequent characters up to the first non-blank, non-tab character following the next newline or the end of the current file, whichever comes first. This convention applies only when reading characters from the file(s) specified on the command line (or the standard input if none is specified) or from an included file. Comments should be removed only once from each file or from standard input. After all inputs are read and comments are removed, then you should start expanding. &nbsp;<em><strong>Note: the comment character can be escaped, see the section below.</strong></em>

<p>&nbsp;</p>

## Escape Characters

Besides being used as the “start” character for a macro, a <code>\\</code> character can also be used to escape one of the following special characters <code>\\</code>, <code>#</code>, <code>%</code>, <code>{</code>, <code>}</code> so that it is not treated as a special character. &nbsp;For these characters, the effect of the <code>\\</code> is preserved until it is about to be output, at which point it is suppressed, and the <code>\\</code>, <code>#</code>, <code>%</code>, <code>{</code>, <code>}</code> is output instead. In effect, the <code>\\</code> is ignored and the following character is treated as a non-special character thereafter. &nbsp;That is, in effect the pair of characters (e.g. <code>\{</code>) can be treated as a macro with no arguments until it is expanded and output. We then have the following cases:

<ul>
  <li>Escape character followed by <code>\</code>, <code>#</code>, <code>%</code>, <code>{</code>, <code>}</code>: &nbsp;for this case use the rule above (i.e., when it is time to output, only print the second character).</li>
  <li>Escape character followed by an alphanumeric character: in this case, we must be reading a macro, so all the macro parsing rules apply.</li>
  <li>Escape character followed by non-alphanumeric and not <code>\</code>, <code>#</code>, <code>%</code>, <code>{</code>, <code>}</code>: &nbsp;in this case, these characters have no special meaning to your parser (i.e., they should both be output).</li>
</ul>

<p>&nbsp;</p>

### Error Detection

The following kinds of errors should be detected:

<ul>
  <li>Parsing Errors
    <ul>
      <li>For example, in a <code>\def</code> macro, if <code>NAME</code> is not a nonempty alphanumeric string.</li>
      <li>Another example would be if a macro name is not immediately followed by an argument wrapped in balanced curly braces</li>
      <li>or more generally: if a macro has too few arguments.</li>
    </ul>
  </li>
  <li>Semantic Errors
    <ul>
      <li>For example, a macro name is not defined.</li>
      <li>Another example would be an attempt to redefine a macro before undefining it</li>
      <li>or an attempt to undefine a nonexistent macro.</li>
      </ul>
  </li>
  <li>Library Errors.
    <ul>
      <li>You should consider how errors may be returned by any library functions you use <!--(<code>malloc</code>, <code>fopen</code>, <code>read</code>, etc.)--> and detect them.
        <!-- <ul>
          <li><em>Note: correct usage of <code>malloc</code>, <code>calloc</code>, etc., should not result in any library errors due to running out of memory.</em></li>
        --> </ul>
      </li>
    </ul>
  </li>
</ul>

For these kinds of errors, your program should write a one-line message to <code>stderr</code> and exit. If you detect an error, you should not output a partial evaluation of any input. The following is a list of errors or scenarios you should ignore:

<ul>
  <li>Cyclical macro definitions</li>
  <li>Cyclical file includes</li>
  <li>Infinite <code>expandafter</code> loops</li>
  <li>Attempts to redefine a built-in macro</li>
</ul>

<p>&nbsp;</p>

### Performance

The number of macros will never be large enough to require more than a linear search of the list of macros. Your program should run in time and space proportional to the number of characters processed (the sum of the lengths of the file(s) specified on the command line, or the standard input if none is specified, and the lengths of all macro expansions). If your program fails any test by exceeding the time or space limit, the burden of proof that this is not an error is on you. The key to good performance here will be on how you handle strings (i.e., your expansion algorithm/data structure), Try to avoid doing string shifts and insertions, as well as Linkedlists at character granularities (although linear, the constant overhead is too high that it will fail performance checks).

<p>&nbsp;</p>

### Misc. Requirements

<ul>
  <li>The input files should be thought of as one long string (after removing comments). Hence, macros defined in the first file should be accessible in the second, and macros can span two or more files.</li>
  <li>Only <strong>safe</strong> Rust is allowed (your code should <strong>not</strong> contain any <strong>unsafe</strong> block).</li>
  <li>Do not use any external libraries other than the Rust Standard Libraries or Core&nbsp; (i.e., do not link any other libraries in your Cargo.toml step).</li>
  <li>The input files should be thought of as one long string (after removing comments). Hence, macros defined in the first file should be accessible in the second, and macros can span two or more files.</li>
  <li>When your program exits, all allocated storage must be reachable.</li>
  <li style="text-align: left;">Your program should have <strong>no warnings (even warning of unused code)</strong> when compiled with:
  <pre style="text-align: center;">cargo build -r</pre>
  </li>
</ul>

<p>&nbsp;</p>

### Style and Code Organization

Your code should be clean, clear, correct, and consistent.  The most important style guideline is consistency. Don’t write code that changes style from line to line. In addition, as a general rule, this assignment will be easier to write if you break up your program into smaller (1-50 lines), reusable functions with readable and unambiguous names.

<p>&nbsp;</p>

## Sample Executions

<ul>
  <li>Circular macro definition to create a list macro.<br><br>
    <table style="border-collapse: collapse; width: 100%; height: 177px;" border="1">
      <tbody>
        <tr>
          <td style="width: 50%;"><strong>Input</strong></td>
          <td style="width: 50%;"><strong>Output</strong></td>
        </tr>
        <tr style="height: 177px;">
          <td style="width: 50%; height: 177px;">
            <pre>\def{list}{\if{#}{#, \list}{..., omega}}%<br>\list{alpha}{beta}{gamma}{}</pre>
          </td>
          <td style="width: 50%; height: 177px;">
            <pre>alpha, beta, gamma, ..., omega</pre>
          </td>
        </tr>
      </tbody>
    </table>
  <br>Explanation:
  <ul>
    <li>The <code>\def</code> defines <code>list</code> to be the string <code>\if{#}{#, \list}{..., omega}</code>.</li>
    <li>The <code>%</code> causes all characters up to but not including the list macro in the second line to be discarded.</li>
    <li>The <code>\list{alpha}</code> is expanded to <code>\if{alpha}{alpha, \list}{..., omega}</code>, so the input is now: <code>\if{alpha}{alpha, \list}{..., omega}{beta}{gamma}{}</code></li>
    <li>Since <code>alpha</code> is not the empty string (it is true), the <code>\if</code> expands to the <code>THEN</code> block, so the input is now alpha, <code>\list{beta}{gamma}{}</code>.</li>
    <li>The <code>alpha,</code> is output to standard out, so the input is now: <code>\list{beta}{gamma}{}</code></li>
    <li>The cycle now repeats for <code>beta</code> and <code>gamma</code> which are both not defined. Which leads to <code>beta</code>, and <code>gamma</code>, being output. After these are expanded the input is now: <code>\list{}</code></li>
    <li>Since the argument is an empty string, the <code>\if</code> expands to <code>omega</code> so that input is now <code>..., omega</code>.</li>
    <li>The <code>..., omega</code> is output, and the program completes.<br><br></li>
  </ul>
  </li>
  <li>More on <code>\expandafter</code><br><br>
    <table style="border-collapse: collapse; width: 100%; height: 177px;" border="1">
      <tbody>
        <tr>
          <td style="width: 50%;"><strong>Input</strong></td>
          <td style="width: 50%;"><strong>Output</strong></td>
        </tr>
        <tr style="height: 177px;">
          <td style="width: 50%; height: 177px;">
            <pre>\def{A}{aardvark}%<br>\expandafter{\def{B}}{{\A{}}}%<br>\undef{A}\def{A}{anteater}%<br>\B{} = \A{}</pre>
          </td>
          <td style="width: 50%; height: 177px;">
            <pre>aardvark = anteater</pre>
          </td>
        </tr>
      </tbody>
    </table>
  <p><br>Explanation:</p>
  <ul>
    <li>The <code>\def</code> defines <code>A</code> to be the string <code>aardvark</code>.</li>
    <li>The <code>%</code> causes all characters up to but not including the <code>\expandafter</code> macro in the following line to be discarded.</li>
    <li>The <code>\expandafter</code> macro causes <code>{\A{}}</code> to be expanded to <code>{aardvark}</code> and then the whole thing is replaced by <code>\def{B}{aardvark}</code>.</li>
    <li>The <code>%</code> causes all characters up to but not including the <code>\undef</code> macro in the following line to be discarded.</li>
    <li>The <code>\undef{A}\def{A}{anteater}</code> causes <code>A</code> to be redefined as <code>anteater</code>.</li>
    <li>The <code>%</code> causes all characters up to but not including the <code>\B</code> macro in the second line to be discarded.</li>
    <li>The <code>\B{}</code> is expanded to <code>aardvark</code> so the remaining input is now <code>aardvark = \A{}</code>.</li>
    <li>The <code>aardvark =</code> is outputted, so the input is now <code>\A{}</code>.</li>
    <li>The <code>\A{}</code> is expanded to <code>anteater</code></li>
    <li><code>anteater</code>&nbsp;is the only remaining text, and no macros remain, so it is output.</li>
  </ul>
  </li>
</ul>
