// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;import java.util.SortedSet;
public class KeyColumnPlainProbe extends KeyColumnProbe {
    public KeyColumnPlainProbe(Book b,SortedSet<Integer> ids){super(b,ids);System.setProperty("native.probe.plainColumn","true");}
}
