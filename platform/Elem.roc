interface Elem
    exposes [
        Elem,
        Color,
        TextModifier,
        BorderModifier,
        BorderType,
        Alignment,
        ScrollOffset,
        Style,
        LayoutDirection,
        Constraint,
        CursorPosition,
        Cursor,
        Corner,
        Span,
        ListConfig,
        BlockConfig,
        ScrollOffset,
        ParagraphConfig,
        LayoutConfig,
        PopupConfig,
        paragraph,
        blockConfig,
        st,
        unstyled,
        layout,
        list,
        styled,
    ]
    imports []

# TODO roc glue can't support more elements in this recusrive tage here yet
Elem : [
    Block BlockConfig,
    Paragraph ParagraphConfig,
    ListItems ListConfig,
    Layout (List Elem) LayoutConfig,
]

## Options to text in a span
##
##     styled "Hello World" { modifiers: [ Bold ] }
TextModifier : [Bold, Dim, Italic, Underlined, SlowBlink, RapidBlink, Reversed, Hidden, CrossedOut]

## Different border display options for a block or widget. Default is `None`.
##
##     blockConfig { borders: [All] }
BorderModifier : [None, Top, Right, Bottom, Left, All]

BorderType : [Plain, Rounded, Double, Thick]

Alignment : [Left, Center, Right]
ScrollOffset : U16
Style : { fg : Color, bg : Color, modifiers : List TextModifier }
LayoutDirection : [Horizontal, Vertical]
Constraint : [Percentage U16, Ratio U32 U32, Length U16, Max U16, Min U16]
CursorPosition : { row : U16, col : U16 }
Cursor : [Hidden, At CursorPosition]
Corner : [TopLeft, TopRight, BottomRight, BottomLeft]
ModalPosition : { percentX : U16, percentY : U16 }
ListSelection : [None, Selected Nat]
PopupConfig : [
    None,
    Centered ModalPosition,
]

unstyled : Str -> Span
unstyled = \str ->
    { text: str, style: defaultStyle }

styled : Str, { fg ?Color, bg ?Color, modifiers ?List TextModifier } -> Span
styled = \str, { fg ? Default, bg ? Default, modifiers ? [] } ->
    { text: str, style: { fg, bg, modifiers } }

st : { fg ?Color, bg ?Color, modifiers ?List TextModifier } -> Style
st = \{ fg ? Default, bg ? Default, modifiers ? [] } -> { fg, bg, modifiers }

defaultStyle = { bg: Default, fg: Default, modifiers: [] }
defaultBlock = {
    title: { text: "", style: defaultStyle },
    titleAlignment: Left,
    style: defaultStyle,
    borders: [],
    borderStyle: defaultStyle,
    borderType: Plain,
}

paragraph :{
        text ? List Line,
        block ? BlockConfig,
        textAlignment ? Alignment,
        scroll ? ScrollOffset,
        cursor ? Cursor,
    }
    -> Elem
paragraph = \{ text ? [], block ? defaultBlock, textAlignment ? Left, scroll ? 0, cursor ? Hidden } ->
    Paragraph {
        text,
        block,
        textAlignment,
        scroll,
        cursor,
    }

## Create a list widget
##
##     title = unstyled "List Items"
##     list { 
##         items : [ 
##             [unstyled "Apple"], 
##             [unstyled "Pear"], 
##             [unstyled "Banana"],
##         ], 
##         selected : Selected 2, 
##         block : blockConfig { title, borders : [ All ] }, 
##         highlightStyle : st { fg : Blue },
##     }
list :{
        items ? List Line,
        selected ? ListSelection,
        block ? BlockConfig,
        style ? Style,
        highlightSymbol ? Str,
        highlightSymbolRepeat ? Bool,
        highlightStyle ? Style,
        startCorner ? Corner,
    }
    -> Elem
list = \{ items ? [],selected ? None,block ? defaultBlock, style ? defaultStyle, highlightSymbol ? ">",highlightSymbolRepeat ? Bool.false, highlightStyle ? defaultStyle,startCorner ? TopLeft,   } -> 
    ListItems { items, selected, block, style, highlightSymbol, highlightSymbolRepeat, highlightStyle, startCorner, }

blockConfig : { title ?Span, titleAlignment ?Alignment, style ?Style, borders ?List BorderModifier, borderStyle ?Style, borderType ?BorderType } -> BlockConfig
blockConfig = \{ title ? { text: "", style: defaultStyle }, titleAlignment ? Left, style ? defaultStyle, borders ? [], borderStyle ? defaultStyle, borderType ? Plain } -> { title, titleAlignment, style, borders, borderStyle, borderType }

# Base widget to be used with all upper level ones.
# It may be used to display a box border around the widget and/or add a title.
BlockConfig : {
    title : Span,
    titleAlignment : Alignment,
    style : Style,
    borders : List BorderModifier,
    borderStyle : Style,
    borderType : BorderType,
}

## Multiple spans on a single line
Line : List Span

## A single line string where all graphemes have the same style
Span : { text : Str, style : Style }

# A widget to display some text
ParagraphConfig : {
    text : List Line,
    block : BlockConfig,
    textAlignment : Alignment,
    scroll : ScrollOffset,
    cursor : Cursor,
}

# Use cassowary-rs solver to split area into smaller ones based on the preferred
# widths or heights and the direction.
LayoutConfig : {
    constraints : List Constraint,
    direction : LayoutDirection,
    vMargin : U16,
    hMargin : U16,
    popup : PopupConfig,
}

# A widget to display several items among which one can be selected (optional)
ListConfig : {
    items : List Line,
    selected : ListSelection,
    block : BlockConfig,
    style : Style,
    highlightSymbol : Str,
    highlightSymbolRepeat : Bool,
    highlightStyle : Style,
    startCorner : Corner,
}

## The following list of 16 base colors are available for almost all terminals;
## - `Light`, `Dark`, `DarkGrey`, `Black`, `Red`, `DarkRed`, `Green`, `DarkGreen`, 
## `Yellow`, `DarkYellow`, `Blue`, `DarkBlue`, `Magenta`, `DarkMagenta` `Cyan`, 
## `DarkCyan`, `White`, `Grey`
## 
## `Indexed` is the [ANSI 256 color](https://jonasjacek.github.io/colors/)
## `Rgb` is supported on most UNIX terminals and Windows 10 only
##
## Underlying library is [crossterm::style::Color](https://docs.rs/crossterm/latest/crossterm/style/enum.Color.html)
Color : [
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb U8 U8 U8,
    Indexed U8,
]

layout : List Elem, { constraints ? List Constraint, direction ? LayoutDirection, vMargin ? U16, hMargin ? U16, popup ? PopupConfig } -> Elem
layout = \children, { constraints ? [], direction ? Vertical, vMargin ? 0u16, hMargin ? 0u16, popup ? None } -> Layout children { constraints, direction, vMargin, hMargin, popup }

