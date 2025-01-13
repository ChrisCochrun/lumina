(slide :background (image :source "~/pics/frodo.jpg" :fit fill)
       (text "This is frodo" :font-size 70))
(slide (video :source "~/vids/test/camprules2024.mp4" :fit contain))
(slide (video :source "~/vids/The magic of Rust's type system.mkv" :fit contain))
(song :id 7 :author "North Point Worship"
      :font "Quicksand Bold" :font-size 60
      :title "Death Was Arrested"
      :background (image :source "file:///home/chris/nc/tfc/openlp/CMG - Bright Mountains 01.jpg" :fit cover)
      :text-alignment center
      :audio "file:///home/chris/music/North Point InsideOut/Nothing Ordinary, Pt. 1 (Live)/05 Death Was Arrested (feat. Seth Condrey).mp3"
      :verse-order (i1 v1 v2 c1 v3 c1 v4 c1 b1 b1 e1 e2)
      (i1 "Death Was Arrested\nNorth Point Worship")
      (v1 "Alone in my sorrow
And dead in my sin

Lost without hope
With no place to begin

Your love made a way
To let mercy come in

When death was arrested
And my life began")
      (v2 "Ash was redeemed
Only beauty remains

My orphan heart
Was given a name

My mourning grew quiet,
My feet rose to dance

When death was arrested
And my life began")
      (c1 "Oh, Your grace so free,
Washes over me

You have made me new,
Now life begins with You

It's Your endless love,
Pouring down on us

You have made us new,
Now life begins with You")
      (v3 "Released from my chains,
I'm a prisoner no more

My shame was a ransom
He faithfully bore

He cancelled my debt and
He called me His friend

When death was arrested
And my life began")
      (v4 "Our Savior displayed
On a criminal's cross

Darkness rejoiced as though
Heaven had lost

But then Jesus arose
With our freedom in hand

That's when death was arrested
And my life began

That's when death was arrested
And my life began")
      (b1 "Oh, we're free, free,
Forever we're free

Come join the song
Of all the redeemed

Yes, we're free, free,
Forever amen

When death was arrested
And my life began

Oh, we're free, free,
Forever we're free

Come join the song
Of all the redeemed

Yes, we're free, free,
Forever amen

When death was arrested
And my life began")
      (e1 "When death was arrested
And my life began

That's when death was arrested
And my life began"))
(song :author "Jordan Feliz" :ccli 97987
      :font "Quicksand" :font-size 80
      :title "The River"
      :background (video :source "~/nc/tfc/openlp/Flood/motions/Brook_HD.mp4" :fit cover)
      :verse-order (v1 c1 v2 c1)
      (v1 "I'm going down to the river")
      (c1 "Down to the river")
      (v2 "Down to the river to pray ay ay!"))
(load "./test_song.lisp")
