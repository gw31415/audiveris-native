// SPDX-License-Identifier: AGPL-3.0-or-later
// Cross-check: no extractor subclass, only read snapshots at original map iteration.
import org.audiveris.omr.sheet.Book;import java.util.SortedSet;
public class KeyBuilderProcessPlainProbe extends KeyBuilderProcessProbe {
    public KeyBuilderProcessPlainProbe(Book b,SortedSet<Integer> ids){super(b,ids);System.setProperty("native.probe.plainExtractor","true");}
}
