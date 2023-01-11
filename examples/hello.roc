app "example-app"
    packages { pf: "https://github.com/lukewilliamboswell/roc-tui/releases/download/0.0.2/WGLVMwEtG9JJbYr60L_HkOqNCcowgkwEhSyqipgisaY.tar.br" }
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
    text = [[Elem.styled model.text { fg: Magenta }]]
    
    [ Elem.layout [ Elem.paragraph { text } ] {} ]
