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
    render : (Model -> List Elem) as Render,
}
programForHost = program

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