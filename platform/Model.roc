interface Model
    exposes [
        Model,
        updateScroll,
    ]
    imports []

# This is a workaround to use glue. The Model should live in the app and be 
# supplied to the platform, but this isn't support by glue yet.

Model : {
    text : Str,
    todos : List Str,
    scroll : U16,
    bounds : { height : U16, width : U16 },
}

updateScroll : Model, [Up, Down, Left, Right] -> Model
updateScroll = \model, direction ->
    scroll = when direction is 
        Up -> Num.subWrap model.scroll 1u16
        Down -> Num.addWrap model.scroll 1u16
        _ -> model.scroll

    {model & scroll : scroll}