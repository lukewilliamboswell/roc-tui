app "example-app"
    packages { pf: "https://github.com/lukewilliamboswell/roc-tui/releases/download/0.0.2/WGLVMwEtG9JJbYr60L_HkOqNCcowgkwEhSyqipgisaY.tar.br" }
    imports [
        pf.Event.{ Event },
        pf.Elem.{ Elem },
    ]
    provides [program, Model] {} to pf

program = { init, update, render }

# Model
Model : {}

init = \_ -> {}

# Handle Events
update : Model, Event -> Model
update = \model, _ -> model

# Render UI
render : Model -> List Elem
render = \_ -> 
    items = [ 
        [Elem.unstyled "Apple"], 
        [Elem.unstyled "Pear"], 
        [Elem.unstyled "Banana"],
    ]
    selected = Selected 2
    highlightStyle = Elem.st { fg : Blue }
    title = Elem.unstyled "List Items"
    block = Elem.blockConfig { title, borders : [All] }
    list = Elem.list { 
        items, 
        selected, 
        block, 
        highlightStyle,
    }

    [ Elem.layout [ list ] {} ]
