// SPDX-License-Identifier: AGPL-3.0-or-later
// Original setPitchRect with controlled odd/even typical heights and real staffs.
import org.audiveris.omr.sheet.Book;import java.util.SortedSet;
public class KeyPitchWindowControlProbe extends KeyBuilderProcessProbe {
    public KeyPitchWindowControlProbe(Book b,SortedSet<Integer> ids){super(b,ids);System.setProperty("native.probe.pitchWindowControl","true");}
}
