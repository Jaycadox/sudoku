StartMS = __systime_ms__()
EndMS = __systime_ms__()
Solved = false

script.display_status_only()
script.on_name(function()
	return "Timer"
end)

script.on_update(function(game)
	if game:is_solved() then
		if not Solved then
			EndMS = __systime_ms__()
			Solved = true
		end
		return ((EndMS - StartMS) .. "ms"), "StatusBarItemOkay"
	else
		return ((__systime_ms__() - StartMS) .. "ms"), "StatusBarItemInProgress"
	end
end)

events.on_board_gen(function(_)
	StartMS = __systime_ms__()
	Solved = false
end)
