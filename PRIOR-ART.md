# Prior Art Reference
Yakui stands on the shoulders of giants.

It takes heavy inspiration from:
* [React] — Declarative UI as a mainstream paradigm
* [Flutter] — Single pass layout model
* [Moxie] — Constructing UI as a tree of topologically-aware functions

[React]: https://reactjs.org/
[Flutter]: https://flutter.dev/
[Moxie]: https://moxie.rs/

## Flutter Layout Algorithm
(from <https://api.flutter.dev/flutter/widgets/Flex-class.html#layout-algorithm>)

> Layout for a Flex proceeds in six steps:
>
> 1. Layout each child a null or zero flex factor (e.g., those that are not Expanded) with unbounded main axis constraints and the incoming cross axis constraints. If the crossAxisAlignment is CrossAxisAlignment.stretch, instead use tight cross axis constraints that match the incoming max extent in the cross axis.
> 2. Divide the remaining main axis space among the children with non-zero flex factors (e.g., those that are Expanded) according to their flex factor. For example, a child with a flex factor of 2.0 will receive twice the amount of main axis space as a child with a flex factor of 1.0.
> 3. Layout each of the remaining children with the same cross axis constraints as in step 1, but instead of using unbounded main axis constraints, use max axis constraints based on the amount of space allocated in step 2. Children with Flexible.fit properties that are FlexFit.tight are given tight constraints (i.e., forced to fill the allocated space), and children with Flexible.fit properties that are FlexFit.loose are given loose constraints (i.e., not forced to fill the allocated space).
> 4. The cross axis extent of the Flex is the maximum cross axis extent of the children (which will always satisfy the incoming constraints).
> 5. The main axis extent of the Flex is determined by the mainAxisSize property. If the mainAxisSize property is MainAxisSize.max, then the main axis extent of the Flex is the max extent of the incoming main axis constraints. If the mainAxisSize property is MainAxisSize.min, then the main axis extent of the Flex is the sum of the main axis extents of the children (subject to the incoming constraints).
> 6. Determine the position for each child according to the mainAxisAlignment and the crossAxisAlignment. For example, if the mainAxisAlignment is MainAxisAlignment.spaceBetween, any main axis space that has not been allocated to children is divided evenly and placed between the children.