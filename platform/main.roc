# UNCOMMENT THIS TO USE ROC PLATFORM
platform "tui"
    requires { Model } { program : _ }
    exposes []
    packages {}
    imports [
        Event.{ Bounds, Event },
        Elem.{ Elem },
        ]
    provides [programForHost]

programForHost : {
    init : (Bounds -> Model) as Init,
    update : (Model, Event -> Model) as Update,
    render : (Model -> [T (List Elem) Model]) as Render,
}
programForHost =
    {
        init: program.init,
        update: program.update,
        render: \model ->
            elems = program.render model
            T elems model
    }

# UNCOMMENT THIS TO USE ROC GLUE
# platform "tui"
#     requires {  } { program : _ }
#     exposes [ Model]
#     packages {}
#     imports [
#         Event.{ Bounds, Event },
#         Elem.{ Elem },
#         ]
#     provides [programForHost]

# Model : {}

# programForHost : {
#     init : (Bounds -> Model) as Init,
#     update : (Model, Event -> Model) as Update,
#     render : (Model -> List Elem) as Render,
# }
# programForHost = program