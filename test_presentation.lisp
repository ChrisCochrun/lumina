(slide :background (image :source "~/pics/frodo.jpg" :fit crop)
       (text "This is frodo" :font-size 50))
(slide (video :source "~/vids/test/chosensmol.mp4" :fit fill))
(song :author "Jordan Feliz" :ccli 97987
      :font "Quicksand" :font-size 80
      :title "The River"
      :background (image :source "./coolbg.jpg")
      (text "I'm going down to the river")
      (text "Down to the river")
      (text "Down to the river to pray ay ay!"))
(song :author "Jordan Feliz" :ccli 97987
      :font "Quicksand" :font-size 80
      :title "The River"
      :background (video :source "./coolerbg.mkv" :fit cover)
      :verse-order (v1 c1 v2 c1)
      (v1 "I'm going down to the river")
      (c1 "Down to the river")
      (v2 "Down to the river to pray ay ay!"))
(load "./10000-reasons.lisp")
