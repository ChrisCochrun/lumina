# <span class="todo TODO">TODO</span> \[#A\] Deployment pipeline and get a MVP going

We are close on this. The main pipeline consists of needing some virtual machines to build the app on windows and macos, but we have versions for both sort of working. We still need the flatpak to be built properly tho.

# <span class="todo TODO">TODO</span> \[#A\] Fix song imports so that they actually get rid of extra cruft

Sometimes a song imported from Genius will have extra junk that was in the middle of the lyrics on the page. Sometimes the lyrics themselves seem to still carry the styling from the webpage and effect the look through the SVG.

??????? Don't know if this is still broke or not…..

# <span class="todo TODO">TODO</span> \[#A\] Find a way to check if an item is in the library on load so that we can import it into the library

# <span class="todo TODO">TODO</span> \[#A\] Good keyboard shortcuts for the song editor so that making songs is faster and more intuitive

# <span class="todo TODO">TODO</span> \[#B\] Add a title/info slide system for songs

This can include title, author, and ccli info so that it will be compliant and helpful. Basically some slides should be generated that show the song info and can be displayed as the song is starting.

# <span class="todo TODO">TODO</span> \[#B\] Add an access time to the database so that we can sort library items by last used or edited.

# <span class="todo TODO">TODO</span> \[#B\] Make audio is song editor able to increase speed so the user can create the song a little faster if they desire.

# <span class="todo TODO">TODO</span> \[#B\] Font in the song editor doesn't always use the original version

There seems to be some issue with fontdb not able to decipher all the versions of some fonts that are OTF and then end up loading the wrong ones in some issues.

# <span class="todo TODO">TODO</span> \[#B\] Find a way to use auth-token in tests for ci

If I can find out how to use my secrets in ci that would free up more tests, but I could also just turn that test off for the CI so that it won't constantly fail for now

# <span class="todo TODO">TODO</span> \[#B\] Saving and loading font awareness

Someday we should make the saving and loading to be aware of the fonts on the system and find a way to embed them into the save file.

# <span class="todo TODO">TODO</span> \[#B\] Video downloading system

We need to create a way for users to download youtube or other videos by URL.

# <span class="todo TODO">TODO</span> \[#B\] Songs should have a way of storing a lyric video or other videos so they can be helpful for the editor

# <span class="todo TODO">TODO</span> \[#B\] Develop ui for settings

# <span class="todo TODO">TODO</span> \[#B\] Develop library system for slides that are more than images or video i.e. content

# <span class="todo TODO">TODO</span> \[#C\] Self signed cert for windows

This was created in the VM on May 10 2026. It is valid for 2 years. Maybe this self signed cert will be ok till we get some reputation, then maybe consider buying a cert or similar.

# <span class="todo TODO">TODO</span> \[#C\] Rename menu actions to menu commands and build a reverse hashmap for settings to map commands to key-binding such that we can allow for remapping them on the fly.

# <span class="todo TODO">TODO</span> \[#C\] Use orgize as a file parser and allow for orgdown files to represent a presentation.

Orgize has some very nice features that will let me determine what things are in an orgdown file and thus take said file and turn it into a presentation.

After looking more and more at how the orgize docs describe things and the testing platform found at: <https://poiscript.github.io/orgize/> I believe this will work. The main things are that I can possibly decide how to interpret certain pieces of orgdown to mean certain things in lumina. Essentially a properties drawer or tag can indicate backgrounds and other info for the slides or songs and then the notes blocks can indicate text that shouldn't be printed into the slide, thus allowing a single orgdown document to illustrate both an entire presentation, but also the notes and plan for the presenter.

I could potentially do the same with markdown, but since this is for me first, I'll use orgdown because I enjoy the syntax a lot more.

# <span class="todo TODO">TODO</span> \[#C\] Allow for a way to split the presentation up with a right click menu for the presentation preview row.

# <span class="todo TODO">TODO</span> \[#C\] Text could be built by using SVG instead of the text element. Maybe I could construct my own text element even

This does almost work. There is a clear amount of lag or rather hang up since switching to the `text_svg` element. I think I may only keep it till I can figure out how to do strokes and shadows in iced's normal text element.

Actually, what if we just made the svg at load/creation time and stored it in the file system for later, then load the entire songs svg's into memory during the presentation to speed things up? Would that be faster than creating them at on the fly? Is it the creation of them that is slow or the rendering?

## SVG performs badly

Since SVG's apparently run poorly in iced, instead I'll need to see about either creating a new text element, or teaching Iced to render strokes and shadows on text.

## Fork Cryoglyph

This fork will render text 3 times. Once for the text, once for the stroke, once for the shadow. This will only be used in the slides and therefore should not be much of a performance hit since we will only be render 3 copies of the given text. This should not be bad performance since it's not a large amount of text.

This also means in our custom widget with our custom fork, we can animate each individually perhaps.

## Actually…..

I tried out a way of generating the svg and rasterizing it ahead of time and then storing it in the file system to be cached. This works out very well. The text is one whole image for a slides text that gets layered on top of the background, but it works out well for now.

The problem with this approach is that every change to a song's text or font metrics means we need to rebuild all the text items for that song. I need to think of a way for the text generation to be done asynchronously so that the ui isn't locked up.

I bet this is tricking up the loading mechanism. Loading only grabs all the backgrounds and audio pieces, not the text<sub>svg</sub> pieces. So maybe it should so that the generator can run again and grab the same pieces from the filesystem rather than recreate them. This gets extra tricky because we may have fonts that are missing when loading a file. In such a case the loading mechanism ought to suggest to the user to grab those fonts and then perhaps load the cached file while being extra clear that any changes will mess up the text since they no longer possess the font that is in the loaded file. Maybe what we can do is during save, save a copy of all the fonts as well and then during load check to see if the computer has them, if they don't offer to install them on the spot such that they can use the font as is. I wonder if we are allowed to pass fonts around that way.

## Made this slightly faster

Since strings are allocated on the heap, I've changed how to construct the svg string a bit, but honestly, it doesn't matter too much because most of the performance cost seems to be in rendering the string using resvg. So, this can still be something that get's fixed later, and I believe that fix will come in the form of a multi-channel signed distance field wgpu rendered text eventually. We can work on this much later though.

# <span class="todo TODO">TODO</span> \[#C\] Make the presenter more modular so things are easier to change. This is vague…

# <span class="done DONE">DONE</span> \[#A\] Make loading not block the UI

Not working just yet but loading is faster.

# <span class="done DONE">DONE</span> \[#A\] Loading and saving need to have a progress indicator of some sort

This is sorta in there, but apparently I'm not streaming the tasks back correctly so there is some sort of disconnect in the UI.

# <span class="done DONE">DONE</span> \[#A\] Custom widget for slides to work with animations

Since position animations are very difficult in Iced, I'm going to be using a custom widget to build my animations, but they still need to have app state to drive them it seems. As to when the animation starts and such.

# <span class="done DONE">DONE</span> \[#A\] Make sure that adding, deleting and editing items in each model is working correctly \[0/0\]

Let's build some tests that ensure that these functions are working for the models. Make sure the models are built in such a way as to make sure that they are testable and work fast for the user.

By making the db functions take the vector of items in the model, we can drain the model, pass an owned version of those items to the async db function(adding, updating, deleting, etc) and then return an updated list of the items back in the Result.

We should probably return a tuple with the original vector of items in case the db function fails somehow. This would be extremely important if we eventually create a server/client architecture and for whatever reason the server fails to respond with an answer, we'd lose all our items.

## <span class="done DONE">DONE</span> \[#A\] Need to test the library

Instead of testing the library itself, I think I'll just create a fake library in each core model and then test it in that

## <span class="done DONE">DONE</span> Move to new design

# <span class="done DONE">DONE</span> \[#A\] Add Action system

This will be based on each slide having the ability to activate an action (i.e. OBS scene switch, OBS start or stop) when it is active.

This is working but the right click context menu is all the way on the edge of the ui so you can't control all the slides. It also needs a lot of help in making the system more robust and potentially lest reliant on the Presenter struct itself.

# <span class="done DONE">DONE</span> \[#A\] Create a view of all slides in a PDF presenation

# <span class="done DONE">DONE</span> \[#A\] Develop DnD for library items

This is limited by the fact that I need to develop this in cosmic. I am honestly thinking that I'll need to build my own drag and drop system or at least work with system76 to fix their dnd system on other systems.

This needs lots more attention

# <span class="done DONE">DONE</span> \[#A\] File saving and loading

Need to make sure we can save a file with all files archived in it and load it back up.

This is giving me a lot of thoughts…

1.  That saving and loading needs to know about fonts as well.
2.  That TextSvgs should likely be saved as well since the other machines may not always have the same fonts.
3.  That means that TextSvg should have a path option that could hold the cached svg that has already been rendered and that this gets changed to the loaded files directory rather than using the default cache directory.

# <span class="done DONE">DONE</span> \[#A\] Add removal and reordering of service<sub>items</sub>

Reordering is finished

# <span class="done DONE">DONE</span> \[#A\] Change return type of all components to an Action enum instead of the Task\<Message\> type \[0%\] \[0/0\]

## <span class="done DONE">DONE</span> Library

## <span class="done DONE">DONE</span> SongEditor

## <span class="done DONE">DONE</span> Presenter

# <span class="done DONE">DONE</span> \[#A\] Need to fix tests now that the basic app is working

Lots of them have been tweaked to be completing now, but there is more work to do and several need to likely be a lot more robust.

Still failing 4 tests, all to do with the db or lisp. I might throw out the lisp code at some point tho. I keep thinking that a better alternative would be to have a markdown serialization system such that you can write slides in markdown somehow and they would be able to be loaded.

# <span class="done DONE">DONE</span> \[#A\] Make sure updating verse updates the lyrics too

[file:~/dev/lumina-iced/src/core/songs.rs::old_verse = verse;](~/dev/lumina-iced/src/core/songs.rs::old_verse = verse;)

This is necessary so that the entire song gets changed and we can propogate those changes then back to the db.

There is likely some work that still needs to be done here, I believe I am somehow deleting some of my verses.

# <span class="done DONE">DONE</span> \[#A\] Need to fixup how songs are edited in the editors

Currently the song is cloned many times to pass around and then finally get updated in DB. Instead, we need to edit the song directly in the editor and after it's been changed appropriatel, run the update<sub>song</sub> method to get the current song and create slides from it and then update it in the DB.

# <span class="done DONE">DONE</span> \[#B\] Build an Animation type that will hold all the info for what a slide animation is.

The animation type that comes with Iced is basically a way to say how long animations take and at what easing to do them, but they do not at all tell you WHAT to animate, that is all in where you put the animation's interpolate function in the view.

So what I think I'll do is either, build a custom widget for slides (might need to do this anyway eventually since we are doing a lot of custom stuff with slides) or build my own Animation type to hold all of the correct info and based on that Animation, place the Iced animation interpolate function where it needs to go.

I need to think more on this. I'm trying to make sure that each slide will always look the way you want it when editing songs, so it's nice to have the `slide_view` function be the authority on what a slide looks like, but it is really difficult to do that well when you also need to make sure that there are special things that happen when it is actually being run in the presentation.

Maybe a good model to follow would be similar the the `gst_video` system we have. To create a settings type that is passed into the function so that when viewing from the presentation vs. editing, you can pass in extra info that will effect the way the function returns and interpolates things…

# <span class="done DONE">DONE</span> Grid mode needs to use the actual aspect ratio correctly for the slide preview

# <span class="done DONE">DONE</span> Song editor audio slider not working

# <span class="done DONE">DONE</span> Preview mode needs to allow for a larger preview of the slide if the library is closed

# <span class="done DONE">DONE</span> Fix the scrolling when switching slides for preview and grid

They both need to be adjusted when changing the size of the slides that are there

# <span class="done DONE">DONE</span> When editing songs, we should ensure that you can't effect the presentation without certain shortcuts

# <span class="done DONE">DONE</span> Fix the right click context menu in service list and library

Remake this just like the one in the preview and grid view probably so that it can work regardless of scroll and things

# <span class="done DONE">DONE</span> Presenter module needs 2 videos

This will allow for us to have different parameters in the framerate and even ensure that we can modify them separately.

# <span class="done DONE">DONE</span> Song Editor has some sort of performance issue.

`core::songs` logs in line 294 whenever even mousing over the song editor.

# <span class="done DONE">DONE</span> \[#B\] Functions for text alignments

This will need to be matched on for the `TextAlignment` from the user

# <span class="done DONE">DONE</span> \[#B\] Songs should have a place to store the audio file and then play it during editing so you can ensure the order of verses

# <span class="done DONE">DONE</span> Move text<sub>generation</sub> function to be asynchronous so that UI doesn't lock up during song editing.

# <span class="done DONE">DONE</span> Build a presentation editor

# <span class="done DONE">DONE</span> Build library to see all available songs, images, videos, presentations, and slides

## <span class="done DONE">DONE</span> Develop ui for libraries

I've got the library basic layer done, I need to develop a way to open the libraries accordion button and then show the list of items in the library

## <span class="done DONE">DONE</span> Need to do search and creation systems yet

# <span class="done DONE">DONE</span> \[#B\] Build editors for each possible item

## <span class="done DONE">DONE</span> Develop ui for editors

# <span class="done DONE">DONE</span> \[#B\] Find a way to load and discover every font on the system for slide building

This may not be necessary since it is possible to create a font using `Box::leak()`.

``` rust
let font = self.current_slide.font().into_boxed_str();
let family = Family::Name(Box::leak(font));
let weight = Weight::Normal;
let stretch = Stretch::Normal;
let style = Style::Normal;
let font = Font {
    family,
    weight,
    stretch,
    style,
};
```

This code creates a font by leaking the Box to a `'static &str`. I just am not sure if the &str stays around in memory after the view function. If it does, then it's not on the stack anymore and should be fine, but if it isn't cleaned up then we will have a memory leak.

Krimzin on Discord told me that maybe the `update` method is a better place for this Box to be created or updated and then maybe I could generate the view from there.

# <span class="done DONE">DONE</span> Build an image editor

# <span class="done DONE">DONE</span> Use Rich Text instead of normal text for slides

This will make it so that we can add styling to the text like borders and backgrounds or highlights. Maybe in the future it'll add shadows too.

# <span class="done DONE">DONE</span> Build a video editor

# <span class="done DONE">DONE</span> Check into `mupdf-rs` for loading PDF's.

# <span class="done DONE">DONE</span> Build Menu

# <span class="done DONE">DONE</span> Find a way for text to pass through a service item to a slide i.e. content piece

This proved easier by just creating the `Slide` first and inserting it into the `ServiceItem`.

# <span class="done DONE">DONE</span> \[#C\] Figure out why the Video element seems to have problems when moving the mouse around

I think this got fixed in a recent update
