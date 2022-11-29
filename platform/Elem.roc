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

Color : [Rgb U8 U8 U8, Default, White, Black, Red, Green, Blue]
TextModifier : [Bold, Dim, Italic, Underlined, SlowBlink, RapidBlink, Reversed, Hidden, CrossedOut]
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
PopupConfig : [
    None,
    Centered ModalPosition
]

unstyled : Str -> Span 
unstyled = \str ->
    { text: str, style: defaultStyle }

styled : Str, { fg ? Color, bg ? Color, modifiers ? List TextModifier } -> Span 
styled = \str, { fg ? Default, bg ? Default, modifiers ? [] } ->
    { text: str, style: { fg, bg, modifiers } }

st : { fg ? Color, bg ? Color, modifiers ? List TextModifier } -> Style
st = \{ fg ? Default, bg ? Default, modifiers ? [] } -> { fg, bg, modifiers }

defaultStyle = { bg: Default, fg: Default, modifiers: [] }

paragraph : {
    text ? List Span,
    block ? BlockConfig,
    textAlignment ? Alignment,
    scroll ? ScrollOffset,
    cursor ? Cursor,
} -> Elem
paragraph = \{ text ? [], block ? {
        title : { text : "", style : defaultStyle },
        titleAlignment : Left,
        style : defaultStyle,
        borders : [],
        borderStyle : defaultStyle,
        borderType : Plain,
    }, textAlignment ? Left, scroll ? 0, cursor ? Hidden } -> 
    Paragraph {
        text,
        block,
        textAlignment,
        scroll,
        cursor,
    }

blockConfig : { title ? Span,titleAlignment ? Alignment,style ? Style,borders ? List BorderModifier,borderStyle ? Style,borderType ? BorderType} -> BlockConfig
blockConfig = \{ title ? { text : "", style : defaultStyle },titleAlignment ? Left,style ? defaultStyle,borders ? [],borderStyle ? defaultStyle,borderType ? Plain} -> {title,titleAlignment,style,borders,borderStyle,borderType}

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

# A single line string where all graphemes have the same style
Span : { text : Str, style : Style }

# A widget to display some text
ParagraphConfig : {
    text : List Span,
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
    items : List Span,
    block : BlockConfig,
    style : Style,
    highlightSymbol : Str,
    highlightSymbolRepeat : Bool,
    highlightStyle : Style,
    startCorner : Corner,
}

layout : (List Elem), {constraints ? List Constraint,direction ? LayoutDirection,vMargin ? U16,hMargin ? U16,popup ? PopupConfig} -> Elem
layout = \children, {constraints ? [],direction ? Vertical,vMargin ? 0u16,hMargin ? 0u16,popup ? None} -> Layout children {constraints,direction,vMargin,hMargin,popup}