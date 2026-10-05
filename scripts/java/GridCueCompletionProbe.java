// SPDX-License-Identifier: AGPL-3.0-or-later
// Original completed GRID on the independent cue raster, with actual staff scale flags.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;
public class GridCueCompletionProbe extends GridFinalProbe {
 final Object book;public GridCueCompletionProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
 public void process(){super.process();try{Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");List<String> states=new ArrayList<>();for(Object st:(List<?>)call(call(sheet,"getStaffManager"),"getStaves"))states.add("{\"staff_id\":"+call(st,"getId")+",\"specific_interline\":"+call(st,"getSpecificInterline")+",\"small\":"+call(st,"isSmall")+",\"line_count\":"+((Collection<?>)call(st,"getLines")).size()+"}");Path p=Path.of(System.getProperty("native.probe.output"));String s=Files.readString(p).trim();Files.writeString(p,s.substring(0,s.length()-1)+",\"staff_states\":["+String.join(",",states)+"],\"native_outputs_used\":false,\"scope\":\"Original completed GRID and persistent staff scale on an independently rendered cue synthetic page; not HEADS/STEMS or published-scan parity.\"}\n");}catch(Exception e){throw new RuntimeException(e);}}
}
