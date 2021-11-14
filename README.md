# Big Mac

This crate contains the branching_parser! metamacro, which can be used to create complex macros with few lines of code.

To use the macro, call it as such (examples for chain_linq's parameters provided)

*Example parser transformation:*

"from x in xs select x * 2" => "from {x} in {xs}, select {x * 2},"

```
branching_parser!{
    @unroll
    path::to::this::module; // (chain_linq;) (note the ; at the end)
    name_of_entrypoint_macro // (linq)
    name_of_parsing_macro // (linq_parser)
    name_of_filter_macro // (linq_filter)
    fully::qualified::destination::macro; // (chain_linq::linq_impl) (note the ; at the end)
    // {series of branch specifiers}
}
```

**Parameters:**

* @unroll is a specifier used to pick the entrypoint syntax of the branching_parser macro.

* The module path is used to set up macro calls correctly. It should be the name of a module that the resulting entrypoint, parser, and filter macros are accessible from 
publicly (e.g. the module you invoked branching_parser! in)

* The 3 names after the path are how you choose the names of the outputted macros. While the parser and filter must be publicly available to consumers of the resulting macro, 
they do not need to be called manually. 

* The destination macro path is the macro to pipe the resulting parsed token stream into; for example, put core::stringify for your macro to ultimately evaluate to a string.

## Rules

Branch specifiers are how the syntax of the resulting macro is decided; They take a branching form that must follow certain rules to create a useful result.

- Each starting branch lives in a new {}.

- Each branch must begin with a token other than #.

- A non-start token can be replaced with # to indicate a string of tokens to be collected.

- A # can be ended with an empty {} to indicate it is terminal.

- A branch can end with a token other than #.

## Examples

For example, LINQ's select statements (of the form select # \[into #\] where \[\] indicates being optional) can be expressed as:

```
{
    select
    {
        ##
        {}
        {
            into
            {
                ## {}
            }
        }
    }
}
```

Another example is LINQ's group by (group # by # \[into #\]):

```
{
    group
    {
        ##
        {
            by
            {
                ##
                {}
                {
                    into
                    {
                        ## {}
                    }
                }
            }
        }
    }
}
```

Finally, an example of a word-terminal chain is LINQ's orderby (orderby # \[ascending | descending\]):

```
{
    orderby
    {
        ##
        {}
        {
            ascending
        }
        {
            descending
        }
    }
}
```

More examples can be found in chain_linq's repo.