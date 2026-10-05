// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.omr.sheet.Book;import java.util.*;
public class BeamsPlainProbe extends BeamsProbe {
 public BeamsPlainProbe(Book b,SortedSet<Integer> ids){super(b,ids);System.setProperty("native.probe.plainBeams","true");}
}
