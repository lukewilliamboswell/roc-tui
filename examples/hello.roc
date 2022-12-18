app "hello-world"
    packages { pf: "../platform/main.roc" }
    imports [
        pf.Event.{ Event },
        pf.Elem.{ Elem },
    ]
    provides [program, Model] {} to pf

program = { init, update, render }

# Model
Model : { text : Str }

init = \_ -> { text: "Hello world!" }

# Handle Events
update : Model, Event -> Model
update = \model, _ -> model

# Render UI
render : Model -> List Elem
render = \model -> 
    text = [[Elem.styled model.text { fg: Green }]]
    
    [ Elem.layout [ Elem.paragraph { text } ] {} ]
