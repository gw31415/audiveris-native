// SPDX-License-Identifier: AGPL-3.0-or-later
// Same original-method controls with no extra graph observation listener.
import org.audiveris.omr.sheet.Book;import java.util.SortedSet;
public class TimeSelectionControlPlainProbe extends TimeSelectionControlProbe {public TimeSelectionControlPlainProbe(Book b,SortedSet<Integer> ids){super(b,ids);System.setProperty("native.probe.plainTimeControl","true");}}
