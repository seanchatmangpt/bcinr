The `bcinr-cheat-scanner` detects circular oracles (`CHEAT-002`) by analyzing the source code's Abstract Syntax Tree (AST).

Here is exactly how the detection mechanism works in `tools/bcinr-cheat-scanner/src/main.rs`:

1. **Function Extraction via AST Visitation**:
   The `SynCheatVisitor` traverses the parsed AST using the `syn` crate. For every function and implementation method (`ItemFn` and `ImplItemFn`), it extracts the function's name and its AST body block. 
   The function's body is converted back into a string using the `quote!` macro, and all whitespace is stripped to normalize it for comparison (`.replace(" ", "")`).
   
   ```rust
   let name = i.sig.ident.to_string();
   let block = &i.block;
   let body_str = quote::quote!(#block).to_string().replace(" ", "");
   self.functions.push((name, body_str));
   ```
   *(Note: It explicitly skips functions or modules annotated with testing or benchmarking attributes)*

2. **Identifying Oracles**:
   After collecting all functions from a file, the `check_circular_oracles` function iterates through them. It specifically looks for any function whose name ends with `_reference` or `_oracle`.

   ```rust
   if name.ends_with("_reference") || name.ends_with("_oracle") {
       let base_name = name
           .trim_end_matches("_reference")
           .trim_end_matches("_oracle");
   ```

3. **Body Comparison**:
   Once an oracle function is identified, the scanner derives its `base_name` (e.g., if the function is `compute_oracle`, the base name is `compute`).
   It then searches the collected functions again for the production function matching this `base_name`.
   If it finds the implementation function, it performs an exact string comparison of their normalized AST bodies (`body == p_body`). 
   If they are identical, it means the oracle is a circular copy of the production implementation, and it raises a `CHEAT-002` violation.

   ```rust
       for (p_name, p_body) in functions {
           if p_name == base_name && body == p_body {
               findings.push(format!(
                   "CHEAT[CHEAT-002]: {} — circular oracle: {} identical to implementation {}",
                   path.display(),
                   name,
                   p_name
               ));
           }
       }
   ```
