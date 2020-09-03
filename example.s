@org 0x200
start:
frame_start:
	cls
	ld I, player_sprite
	drw va, vb, 3

	ld v8, dt
	sne v8, 4
	add vb, 1

	ld v6, 1
	ld v7, 0x8
	sknp v6
	call player_jump
	jmp frame_start

player_jump:
	ld v3, dt
	se v3, 0
	jmp dont_jump

	ld v3, 30
	ld dt, v3

	sub vb, v7
dont_jump:
	ret


player_sprite: 
@db 0xe0, 0xe0, 0xe0