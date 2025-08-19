# break front wall
let 5 # shout
copy out reg0
# move to center
let 1 # move forward
copy out reg0
copy out reg0
copy out reg0
copy out reg0
copy out reg0
copy out reg0
let 0 # trun left
copy out reg0

@loop:
let 5 # shout
copy out reg0
copy out reg0
copy out reg0
copy out reg0
let 0 # trun left
copy out reg0
copy out reg0
@wait:
let 4
copy out reg0
copy reg3 in
let @wait
eq # if fornt is empty
let @loop
on

