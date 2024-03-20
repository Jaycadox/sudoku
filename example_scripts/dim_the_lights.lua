-- Dim the lights
events.on_update(function(_game)
    width, height = drawing.game_size()
    x, y = drawing.game_origin()
    drawing.draw_rect(x, y, width, height, 0.0, 0.0, 0.0, 0.8)
end)