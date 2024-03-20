-- Auto find easy boards
events.on_board_gen(function(game)
    small_count = 0
    for i, v in pairs(game:cells()) do
        if v == 0 and #(game:unoccupied_cells_at(i - 1)) == 1 then
            small_count = small_count + 1
        end
    end
    if small_count < 7 then
        game:enter_buffer_command("BoardGen")
    else
        game:enter_buffer_command("Eval \"Found with " .. small_count .. " free\"")
        events.wait_ms(800, function(game)
            game:enter_buffer_command("Eval \"\"")
        end)
    end
end)

