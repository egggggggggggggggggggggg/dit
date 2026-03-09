TODO 
- Make glyph atlas more ergonomic in that it's less hard coded. Currently all the values are hardcoded.
- Possible changes: Define a set of Unicode Ranges for easier selection of neeeded characters. 
- Create a Font struct that abstracts the messy aspects of individual font file types lik ttf, otf, etc,
- The Atlas struct or another struct should serve as the entry point for anything glyph related instead of two seperated things requiring the other to be passed into it to do stuff.
- Create a cache for glyphs to limit excessive GPU memory usage for texture atlas uploading. 
- Finish the ANSI Escape Code Parser and implement Handler for screen

- Optimize graphics - This one is more so research. Possible optimizations: Instancing.
- Define a Renderable trait? Makes it easier to extend the API for usage in other graphic settings.
