/* FUNCTION PREDICTION
   ===================

This code is used to guess what labels are functions, and their
arguments and return values. Although we could force the user to
specify this information, it is ideal to guess it via code metadata.

In the future, we can use both this information and the user's
information to make a more accurate prediction.

   OUTLINE
   =======
1. Determine functions by looking for labels that are called.
If a label is both called to and jumped to, add an error
2. Get INs and OUTs of all function entry-points and exit-points.
   We set the

-
*/

use crate::cfg::CFG;

impl CFG {}
