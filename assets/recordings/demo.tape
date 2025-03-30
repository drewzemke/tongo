Output assets/demo.gif
Set Width 1600
Set Height 900
Set Theme "Dracula"
Set Framerate 24

# open the app
Type "tongo"
Sleep 1s
Enter
Sleep 1s

# select connection
Down 1
Sleep 500ms
Enter
Sleep 1s

# select db
Down 1
Sleep 50ms
Down 1
Sleep 50ms
Down 1
Sleep 500ms
Enter
Sleep 1s

# select connection
Down 1
Sleep 500ms
Up 1
Sleep 500ms
Enter
Sleep 1s

# expand and collapse doc
Left 1
Sleep 1s
Right 1
Sleep 1s

# edit the filter
Type "K"
Sleep 500ms
Enter
Backspace
Sleep 500ms
Type " age: { $gt: 27 } }"
Sleep 1s
Enter
Sleep 3s

# select the second doc
Down 7
Sleep 1s

# open the doc in editor
Type "E"
Sleep 2s
Type "6j"
Sleep 1s
Type "o"
Sleep 500ms
Type `"is_cool": true,`
Escape
Sleep 1s
Type ":wq"
Sleep 500ms
Enter
Sleep 2s

# observe the result
Down 5
Sleep 4s

