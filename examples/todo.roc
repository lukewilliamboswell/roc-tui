app "example-app"
    packages { pf: "../platform/main.roc" }
    imports [
        pf.Event.{ Event, Bounds },
        pf.Elem.{ Elem },
    ]
    provides [program, Model] {} to pf

program = { init, update, render }

# Model
Model : { draft : Str, todos : List Str, selected : Nat }

init : Bounds -> Model
init = \_ -> { draft: "", todos: [], selected : 0 }

# Handle Events
update : Model, Event -> Model
update = \model, event ->
    when event is
        KeyPressed code ->
            when code is
                Scalar char -> { model & draft: Str.concat model.draft char, selected : 0 }
                Delete | Backspace -> { model & draft: removeChar model.draft }
                Up -> 
                    if model.selected > 0 then 
                        {model & selected : model.selected - 1}
                    else 
                        model 
                Down -> 
                    if model.selected < List.len model.todos then 
                        {model & selected : model.selected + 1}
                    else 
                        model
                Enter ->
                    if model.selected == 0 && Str.isEmpty model.draft == Bool.false then 
                        { model & draft: "", todos : List.append model.todos model.draft }
                    else if model.selected > 0 then
                        {model & todos : List.dropAt model.todos (model.selected - 1), selected : 0}
                    else 
                        model

                _ -> model

        _ ->
            model

removeChar = \text ->
    textUtf8 = Str.toUtf8 text |> List.dropLast
    when Str.fromUtf8 textUtf8 is 
        Ok str -> str
        Err _ -> text 

# Render UI
render : Model -> List Elem
render = \model ->
    [
        Elem.layout
            [ inputBox model.draft, renderTodos model.todos model.selected ]
            { constraints: [Min 3, Ratio 1 1] }
    ]

inputBox = \draft ->
    title = Elem.unstyled "What do you need todo?"
    block = Elem.blockConfig { title, borders: [All], borderStyle: Elem.st { fg: Blue } }
    text = 
        if Str.isEmpty draft then 
            [[Elem.styled "Type your todo here... and press enter to save it." { fg : Red}]]
        else 
            [[Elem.styled draft { bg: Black, fg: White }]]
    cursor = 
        if Str.isEmpty draft then 
            Hidden
        else 
            col = Str.countGraphemes draft |> Num.toU16 |> Num.add 1
            At { row : 1, col }

    Elem.paragraph { block, text, cursor }

renderTodos = \todos, selected ->
    text = 
        todo, index <- List.mapWithIndex todos
        i = Num.toStr (index + 1)

        if selected == (index + 1) then 
            [
                Elem.styled "#\(i): " { bg: Black, fg: White },
                Elem.styled todo { fg: Green },
            ]
        else 
            [
                Elem.unstyled "#\(i): ",
                Elem.unstyled todo,
            ]
    title = Elem.unstyled "TODOs"
    block = Elem.blockConfig { title, borders: [All] }

    Elem.paragraph { text, block }

