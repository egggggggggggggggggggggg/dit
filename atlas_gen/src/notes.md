What to know

Parametric equations
Represent multiple values that aren't dependent on each other but the same indepndent value
ex: Bezier curve where the points are the undependents and t the value between [0, 1] that gives the result

Bezier Curves and their general formula
Linear interpolation with multiple points
Derivative of said curve can be used to calculate direction vector
Derirative of the function where T = 1 or 0 to get the important vector
With multiple direction vectors obtained you can calculate if theres a sharp corner where two bezier curves meet at an endpoint
Cross product of these two direction vectors = sharpness aka how much of a corner is it
Normally cross product returns a vector but here it returns a psuedoscalar
Cross product = axby - aybx
For any two parallel vectors the result is 0
if u get the sign of the result it can be used to tell if the vectors are in clockwise order 
This is relevant cuz ???

Floating point arithmetics obviously has precision errors
hence the usage of a cut off / threshold value
threshold value is defined as sin of alpha where alpha should be less than or equal to pi
3 radians was used in msdf-atlas-gen
this gives 0.14112... when plugged into sin(alpha)
For some reason because of this thing you gotta normalize the direction vectors
magnitude(normalize(vecA) * normalize(vecB)) <= sin(alpha)

Something about cross product being zero but 180 degree turn 
accounting for that u check dot product of direction vectors to ensure they're positive
dot(vecA, vecB) > 0

Distance stuff now

General formula for a bezier curve to a point is  
d/dt||b(t) - p|| = 0
(b(t) - p) * d/dtb(t) = 0
think of p as offseting the equation turning it into a polynomial where all you have to do is find the roots
derirative = tangent line of polynomial, basically asking what tan line = 0 or roots. 
for a given value t between 0, 1, 


Line segment case
when simplfied t = ( (p - p0) * (p1 - p0) ) / ( (p1 - p0) * (p1 - p0) )
check if t lies in the range 0, 1. if not remove it and clamp to 0 or 1 

Other cases just reduce down to a polynomial and solve for roots of said polynomial
smth about auxillary vevctors for simplification 
evaluate the roots between 0, 1 and those range values themselves and take the min from them
thats the min dist

Signed

Determining which side P is on
use cross product for this
sgn(d/dtb(t) * b(t) - p)||b(t) - p ||
*redudant calc in thesis idk

Point to a shape distance

Same distance idea applies but issues arise with signage
A given point can be equidistant to two segments and the min dist
would be to the corner S where the segments meet
to solve this the two segments can be dividing on a plane between the two segments
Said plane can  be though of as the measure of orthagnality/how perpendicular it is to a given segment
Formula in the thesis paper

Pseudo-distance fields
idk why this is important
Allows line segments to extend inifinitely outward
For curves t can be continued or just a line segment extending from the direction vector of its endpoints
Fails in certain cases so its not as useful

Shape simplifying
Preprocess the shape to make it easier to perserve only important stuff
Certain edges may be close together
During pre process of the edges prune any that are shorten than a given limit
Reconnect them by moving the end points of eges together
Some other stuff idk

Corners

In sdf corners fail cuz they dont have enough samples to go off
leads to sharp corners getting rounded

Another case where two sharp corner edges meet 
When that happens its random as to whether theres rounding or not
Depends on grid alignment
Proves a corner can be partitioned into 8 areas
Four quadrants = A,B,C,D 
Prime letter quadrants like A' are the inner curved quadrants
for a given prime quadrant its value (bool) can be derived from a bool function
that takes in the quadrants and a random bit r
n() = not
+ = or 
for A' its f(a,b,c,d,r) = ab + ac + bcd + r(an(bc)d) + ...

Binary vectors and smth

0 with arrow = vec filled with 0 
vice versa for 1 with arrow


padding of the areas

Color strat
let there be two vectors I and O 
I  must have two of its field set  
eg: (0,1,1)
O must have 1 of its field set 
eg: (0,1,0)
These two vectors must satisfy the property that I&O does not equal (0,0,0)
For each corner detected

Classify if convex or concave

If convex: 
    Refer to the diagram that labels convex in paper
If concave: 
    Refer to the diagram that labels concave in paper
Pitfall in that it cannot account for a certain case 

Collision of uncorrelated areas
???


Add padding is the fix apparently
to derive the padding or the binary vectors of A  B if its inside the shape
if its outside the shape and the binary vectors of A and B 
If a and b then no padding so its fine if the operation is done regardless 

Implementing the idea 

Edge pruning 
Couldn't find a specified threshold value for this personal testing prob

General Image plotting stuff
The pixel cords must be centerd so add  1/2 to the x,y coordinates, divide by width and height 
to get the point P 

preliminary defines
let dMax = sm value (typically 255 or 1)
let range = -dMax,  dMax 
said range will be xonverted to 0, 255 aka normalized for texture atlas 
for a given distance the color can be acquired by doing
distanceRange = 2 * dMax
maxColor = 255 for byte bitmaps
( (distance / distanceRange) + 1/2 ) * maxColor 

Signed distance - do the ortho + regular distance comps to properly get the right signed distance  
some stuff 

Edge coloring
Rules: 
Two channels must be on (I vector)
resulting colors left: white, yellow, magenta, cyan
corner sharpness perservation = two adjacent edges must have only one channel in common
algorithm described in 6 

To use the MSDF texture atlas
define the bilinear sampling method(built in on most gpus)
colordist function defined earlier

