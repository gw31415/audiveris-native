// SPDX-License-Identifier: AGPL-3.0-or-later
// Controlled start-attribute injection after an actual raster projector process.
// Does not claim that detectStartColumns naturally produces this state.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;
public class GridLineRootProbe extends GridProbe {
 private final Object book;public GridLineRootProbe(Book b,SortedSet<Integer>ids){super(b,ids);book=b;}
 public void process(){try{
 Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");Object builder=Class.forName("org.audiveris.omr.sheet.grid.GridBuilder").getConstructor(Class.forName("org.audiveris.omr.sheet.Sheet")).newInstance(sheet);Object lr=get(builder,"linesRetriever");call(lr,"createBothLags");call(lr,"retrieveLines");call(lr,"addShortSections");Object staff=((List<?>)call(call(sheet,"getStaffManager"),"getStaves")).get(0);
 Object pr=Class.forName("org.audiveris.omr.sheet.grid.StaffProjector").getConstructor(Class.forName("org.audiveris.omr.sheet.Sheet"),Class.forName("org.audiveris.omr.sheet.Staff"),Class.forName("org.audiveris.omr.sheet.grid.PeakGraph")).newInstance(sheet,staff,null);call(pr,"process");Object first=((List<?>)call(pr,"getPeaks")).get(0),side=Class.forName("org.audiveris.omr.util.HorizontalSide").getField("LEFT").get(null);
 List<String>blanks=new ArrayList<>();for(Object blank:(List<?>)get(pr,"allBlanks"))blanks.add("["+get(blank,"start")+","+get(blank,"stop")+"]");
 List<String>cases=new ArrayList<>();
 for(boolean inject:List.of(false,true)){
  int initial=(Integer)call(first,"getStop");call(staff,"setAbscissa",side,initial);if(inject)call(first,"setStaffEnd",side);
  boolean before=(Boolean)call(first,"isStaffEnd",side);call(pr,"checkLinesRoot");
  cases.add("{\"has_start\":"+before+",\"has_brace\":false,\"first_start\":"+call(first,"getStart")+",\"initial_left\":"+initial+",\"final_left\":"+call(staff,"getAbscissa",side)+",\"final_has_start\":"+call(first,"isStaffEnd",side)+"}");
 }
 Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"provenance\":\"controlled start attribute after raster projector; not a natural GRID branch\",\"interline\":"+call(call(sheet,"getScale"),"getInterline")+",\"blanks\":["+String.join(",",blanks)+"],\"cases\":["+String.join(",",cases)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
