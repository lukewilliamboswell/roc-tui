app "example-app"
    packages { pf: "../platform/main.roc" }
    imports [
        pf.Event.{ Event, Bounds },
        pf.Elem.{ Elem },
    ]
    provides [program, Model] {} to pf

program = { init, update, render }

# Model
Model : {showPopup : Bool}

init : Bounds -> Model
init = \_ -> {showPopup : Bool.true}

# Handle Events
update : Model, Event -> Model
update = \model, event ->
    when event is
        KeyPressed code ->
            when code is
                Enter ->
                    {model & showPopup : !model.showPopup}
                _ -> model

        _ -> model

# Render UI
render : Model -> List Elem
render = \model ->
    when model.showPopup is 
        c if c -> [ body, modal ]
        _ -> [ body ]

# Background widgets
bgText = [[Elem.styled "Some background text... press Enter key to toggle modal!" { fg: Blue }]]
title = Elem.unstyled "Popup Demo"
block = Elem.blockConfig { title, borders : [All] }
body = Elem.layout [ Elem.paragraph { text : bgText, block } ] {}

# Popup modal
fgText = [[Elem.styled "Some foreground text... press Enter to close me!" { fg: Green }]]
modal = 
    Elem.layout [ 
        Elem.paragraph { text : fgText, block, textAlignment : Center },
    ] { 
        popup : Centered { percentX : 50, percentY : 50 },
    }