// SPDX-License-Identifier: AGPL-3.0-or-later
// Explicit controlled slice overlap, observed by the original getSlicePixels.
import org.audiveris.omr.sheet.Book;
import java.util.SortedSet;
public class KeyNeighborEraseControlProbe extends KeyFallbackProbe {
    public KeyNeighborEraseControlProbe(Book book, SortedSet<Integer> ids) {
        super(book, ids);
        System.setProperty("native.probe.neighborControl", "true");
    }
}
