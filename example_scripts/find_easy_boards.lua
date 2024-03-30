-- Auto find easy boards
Active = false

script.on_name(function()
	return "FindEasy"
end)
script.display_name_only()

script.on_activate(function(game, _buffer)
	Active = true
	game:enter_buffer_command("BoardGen")
end)

script.on_update(function(_game)
	if Active then
		return "", "StatusBarItemInProgress"
	else
		return "", "StatusBarItemOkay"
	end
end)

events.on_board_gen(function(game)
	if not Active then
		return
	end

	local small_count = 0
	for i, v in pairs(game:cells()) do
		if v == 0 and #(game:unoccupied_cells_at(i - 1)) == 1 then
			small_count = small_count + 1
		end
	end
	if small_count < 7 then
		game:enter_buffer_command("BoardGen")
	else
		Active = false
		game:enter_buffer_command('Eval "Found with ' .. small_count .. ' free"')
		events.wait_ms(800, function(game)
			game:enter_buffer_command('Eval ""')
		end)
	end
end)
