TO DO: 
    Config parser that returns state and also OS-dependent stuff like font files for the parser
        The parsing stuff isn't bad 
        OS stuff might be a bit hard tho
    Rewrite the code to be more clean and easier to adjust
        This is mainly just the vulkan code but everythings basically baked in
    idk
    Fix the artifacts with the atlas generator
    Actually hook it up to a shell
    Improve the rendering capabilites
        Limit the FPS to monitor refresh rate
        Hook in keyboard events(typing) into the screen so letters show up
        Requires dynamic vertex buffer stuff (staging + host buffer)
        Add more customizablity to the text rendering via uniforms(undercurls, text color, bg, fg, etc.)
    Optimize
        Mainly just removing dumb ways of doing stuff (file parsing maybe)
        Cooler optimizations like multithreading if needed
    Terminal Logic
        Cursor 
        ANSI/VT escape character parser
        Screen grid + scrollback
        History
    Customizability
        This is after everything else is done 
        Stuff like a custom api for integrating add-ons whatnot ig
    
DONE: 
    Basic font parser(ttf only tho)
    Semi-functional msdf texture atlas generator
    Boilerplate vulkan code with a simple window set up
    Semi functional keyboard listener