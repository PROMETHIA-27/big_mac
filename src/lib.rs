#![feature(trace_macros)]

#[macro_export]
macro_rules! define_filter {
    (
        $macro_parser:ident
        $macro_filter:ident
        [$($word:tt)+]
        $ds:tt
    ) => {
        #[macro_export]
        macro_rules! $macro_filter {
            $(
                (
                    { $ds ($ds state:tt)* }
                    $word
                ) => {
                    $macro_parser! {
                        $ds ($ds state)*
                        {HALT $word}
                    }
                };
            )+

            (
                {$ds ($ds state:tt)* }
                $ds tok:tt
            ) => {
                $macro_parser! {
                    $ds ($ds state)*
                    {$tok}
                }
            };
        }
    }
}

#[macro_export]
macro_rules! branching_parser {
    //Archetype:
    //{Completed patterns} 
    //{Queue to complete} as {...queued info...} so each one is a single token tree
    //{Dequeued}
    //Macro_name
    //Filter_name
    //Final_call_name
    //$

    // Once all patterns computed, finalize
    (
        { $($patterns:tt)* }
        {}
        {}
        $module_path:path;
        $macro_name:ident
        $macro_parser:ident
        $macro_filter:ident
        $macro_call:path;
        $ds:tt
    ) => {
        #[macro_export]
        macro_rules! $macro_parser {
            $($patterns)*

            // Completed munching; pass to target
            (
                @$ds tag:tt
                {{$ds ($ds archive:tt)*}}
                {}
            ) => {
                $macro_call!($ds ($ds archive)*)
            };

            // Catch the case where tokens run out, so it doesn't infinitely recurse on the setup
            (
                @$ds tag:tt
                {$ds ($ds archive_working:tt)*}
                {}
            ) => {
                compile_error!("Ran out of tokens before finishing current token chain!");
            };
        }

        #[macro_export]
        macro_rules! $macro_name {
            // Setup munching archetype
            ($ds ($ds toks:tt)+) => {
                {
                    use $module_path::{$macro_parser, $macro_filter};

                    $macro_parser! {
                        @()
                        {{}}
                        {$ds ($ds toks)+}
                    }
                }
            };
        }
    };

    // End chain muncher for terminal word
    (
        { $($patterns:tt)* }
        { $($queue:tt)* }
        {
            @($prev_tag:tt #) $word:tt
        }
        $module_path:path;
        $macro_name:ident
        $macro_parser:ident
        $macro_filter:ident
        $macro_call:path;
        $ds:tt
    ) => {
        $crate::branching_parser!{
            {
                (
                    @$prev_tag
                    {{$ds ($ds archive:tt)* } {$ds ($ds working:tt)*}}
                    {$word $ds ($ds toks:tt)*}
                ) => {
                    $macro_parser! {
                        @()
                        {{$ds ($ds archive)* {$ds ($ds working)*}} $word , }
                        {$ds ($ds toks)*}
                    }
                };

                (
                    @$prev_tag
                    {{$ds ($ds archive:tt)* } {$ds ($ds working:tt)*}}
                    {$ds ($ds toks:tt)*}
                    {$word}
                ) => {
                    $macro_parser! {
                        @()
                        {{$ds ($ds archive)* {$ds ($ds working)*} $word , }}
                        {$ds ($ds toks)*}
                    }
                };

                $($patterns)*
            }
            {$($queue)*}
            {}
            $module_path;
            $macro_name
            $macro_parser
            $macro_filter
            $macro_call;
            $
        }
    };

    // End chain muncher for halt
    (
        { $($patterns:tt)* }
        { $($queue:tt)* }
        {
            @($prev_tag:tt #)
        }
        $module_path:path;
        $macro_name:ident
        $macro_parser:ident
        $macro_filter:ident
        $macro_call:path;
        $ds:tt
    ) => {
        $crate::branching_parser!{
            {
                // Handle HALT callback
                (
                    @$prev_tag
                    {{$ds ($ds archive:tt)* } {$ds ($ds working:tt)*}}
                    {$ds ($ds toks:tt)*}
                    {HALT $ds halt_tok:tt}
                ) => {
                    $macro_parser! {
                        @()
                        {{$ds ($ds archive)* {$ds ($ds working)*} ,}}
                        {$ds halt_tok $ds ($ds toks)*}
                    }
                };

                // Handle valid token callback
                (
                    @$prev_tag
                    {{$ds ($ds archive:tt)* } {$ds ($ds working:tt)*}}
                    {$ds next_tok:tt $ds ($ds toks:tt)*}
                    {$ds tok:tt}
                ) => {
                    $macro_filter! {
                        {
                            @$prev_tag
                            {{$ds ($ds archive)*} {$ds ($ds working)* $tok}}
                            {$ds ($ds toks)*}
                        }
                        $ds next_tok
                    }
                };

                // Handle last token callback
                (
                    @$prev_tag
                    {{$ds ($ds archive:tt)* } {$ds ($ds working:tt)*}}
                    {}
                    {$ds tok:tt}
                ) => {
                    $macro_parser! {
                        @()
                        {{$ds ($ds archive)* {$ds ($ds working)* $tok} ,}}
                        {}
                    }
                };
    
                // Begin calling filter
                (
                    @$prev_tag
                    {{$ds ($ds archive:tt)* } {$ds ($ds working:tt)*}}
                    {$tok:tt $ds ($ds toks:tt)*}
                ) => {
                    $macro_filter! {
                        {
                            @$prev_tag
                            {{$ds ($ds archive)* } {$ds ($ds working)* }}
                            {$ds ($ds toks)*}
                        }
                        $tok
                    }
                };

                $($patterns)*
            }
            {$($queue)*}
            {}
            $module_path;
            $macro_name
            $macro_parser
            $macro_filter
            $macro_call;
            $
        }
    };

    // End chain muncher for word 
    (
        { $($patterns:tt)* }
        { $($queue:tt)* }
        {
            @($prev_tag:tt #)
            $word:tt
            $(
                { $($branches_toks:tt)* }
            )+
        }
        $module_path:path;
        $macro_name:ident
        $macro_parser:ident
        $macro_filter:ident
        $macro_call:path;
        $ds:tt
    ) => {
        $crate::branching_parser!{
            {
                (
                    @$prev_tag
                    {{$ds ($ds archive:tt)* } {$ds ($ds working:tt)*}}
                    {$word $ds ($ds toks:tt)*}
                ) => {
                    $macro_parser! {
                        @(($prev_tag #) $word)
                        {{$ds ($ds archive)* {$ds ($ds working)*} $word }}
                        {$ds ($ds toks)*}
                    }
                };

                $($patterns)*
            }
            {
                $({ @(($prev_tag #) $word) $($branches_toks)* })+

                $($queue)*
            }
            {}
            $module_path;
            $macro_name
            $macro_parser
            $macro_filter
            $macro_call;
            $
        }
    };

    // Set up chain muncher
    (
        { $($patterns:tt)* }
        { $($queue:tt)* }
        {
            @$this_tag:tt 
            #
            $(
                { $( $next_word:tt $($branches_toks:tt)* )? }
            )+
        }
        $module_path:path;
        $macro_name:ident
        $macro_parser:ident
        $macro_filter:ident
        $macro_call:path;
        $ds:tt
    ) => {
        $crate::branching_parser!{
            {
                (
                    @$this_tag
                    {{$ds ($ds archive:tt)* } {$ds ($ds working:tt)*}}
                    {$ds tok:tt $ds ($ds toks:tt)*}
                ) => {
                    $macro_parser!{
                        @$this_tag
                        {{$ds ($ds archive)*} {$ds ($ds working)* $ds tok}}
                        {$ds ($ds toks)*}
                    }
                };
                
                (
                    @$this_tag
                    {{$ds ($ds archive:tt)* }}
                    {$ds ($ds toks:tt)*}
                ) => {
                    $macro_parser!{
                        @$this_tag
                        {{$ds ($ds archive)* } {}}
                        {$ds ($ds toks)*}
                    }
                };

                $($patterns)*
            }
            {
                $({ @($this_tag #) $( $next_word $($branches_toks)* )? })+

                $(
                    $({ !branch_selector @$this_tag $next_word ($($branches_toks)*) })?
                )+
                
                $($queue)*
            }
            {} 
            $module_path;
            $macro_name
            $macro_parser
            $macro_filter
            $macro_call;
            $
        }
    };

    // Set up branch selector for token muncher; in the case of a branching path at a muncher, this will prefer to move to the defined word rather than collect it as a token
    (
        { $($patterns:tt)* }
        { $($queue:tt)* }
        { 
            !branch_selector 
            @$this_tag:tt
            $next_word:tt
            ($($branches_toks:tt)+)
        }
        $module_path:path;
        $macro_name:ident
        $macro_parser:ident
        $macro_filter:ident
        $macro_call:path;
        $ds:tt
    ) => {
        $crate::branching_parser!{
            {
                (
                    @$this_tag
                    {{ $ds ($ds archive:tt)* } { $ds ($ds working:tt)* }} 
                    { $ds ($ds toks:tt)* }
                    {$next_word}
                ) => {
                    $macro_parser! {
                        @(($this_tag #) $next_word)
                        {{$ds ($ds archive)* {$ds ($ds working)*} $next_word } {}}
                        {$ds ($ds toks)*}
                    }
                };

                $($patterns)*
            }
            { $($queue)* }
            {}
            $module_path;
            $macro_name
            $macro_parser
            $macro_filter
            $macro_call;
            $
        }
    };

    // Handle the branch selector case where there is actually no following branch tokens, meaning it is a terminal and should not emit this pattern
    (
        { $($patterns:tt)* }
        { $($queue:tt)* }
        { 
            !branch_selector 
            @$this_tag:tt
            $next_word:tt
            ()
        }
        $module_path:path;
        $macro_name:ident
        $macro_parser:ident
        $macro_filter:ident
        $macro_call:path;
        $ds:tt
    ) => {
        $crate::branching_parser!{
            { $($patterns)* }
            { $($queue)* }
            {}
            $module_path;
            $macro_name
            $macro_parser
            $macro_filter
            $macro_call;
            $
        }
    };

    // Set up word muncher
    (
        { $($patterns:tt)* }
        { $($queue:tt)* }
        {
            @$this_tag:tt 
            $word:tt
            $(
                { $($branches_toks:tt)* }
            )+
        }
        $module_path:path;
        $macro_name:ident
        $macro_parser:ident
        $macro_filter:ident
        $macro_call:path;
        $ds:tt
    ) => {
        $crate::branching_parser!{
            {
                (
                    @$this_tag
                    {{$ds ($ds archive:tt)* } }
                    {$word $ds ($ds toks:tt)*}
                ) => {
                    $macro_parser!{
                        @($this_tag $word)
                        {{$ds ($ds archive)* $word}}
                        {$ds ($ds toks)*}
                    }
                };

                $($patterns)*
            }
            {
                $(
                    { @($this_tag $word) $($branches_toks)* }
                )+

                $($queue)*
            }
            {}
            $module_path;
            $macro_name
            $macro_parser
            $macro_filter
            $macro_call;
            $
        }
    };

    //Dequeue
    (
        {$($patterns:tt)*}
        {$dequeue:tt $($queue:tt)*}
        {}
        $module_path:path;
        $macro_name:ident
        $macro_parser:ident
        $macro_filter:ident
        $macro_call:path;
        $ds:tt
    ) => {
        $crate::branching_parser!{
            {$($patterns)*}
            {$($queue)*}
            $dequeue
            $module_path;
            $macro_name
            $macro_parser
            $macro_filter
            $macro_call;
            $
        }
    };

    // Initialize empty attributes and set up structure
    (
        @unroll
        $module_path:path;
        $macro_name:ident
        $macro_parser:ident
        $macro_filter:ident
        $macro_call:path;
        $(
            {$word:tt $($branch_contents:tt)+}
        )+
    ) => {
        $crate::define_filter! {
            $macro_parser
            $macro_filter
            [$($word,)+]
            $
        }

        $crate::branching_parser!{
            {}
            { $({ @() $word $($branch_contents)+ })+ }
            {}
            $module_path;
            $macro_name
            $macro_parser
            $macro_filter
            $macro_call;
            $
        }
    };
}