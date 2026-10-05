// SPDX-License-Identifier: AGPL-3.0-or-later
// Original GRID/HEADERS/STEM_SEEDS and actual SpotsBuilder header erasure.
import org.audiveris.omr.sheet.Book;import java.util.*;import java.nio.file.*;
public class BeamDispatchPixelsProbe extends BeamsProbe {
 public BeamDispatchPixelsProbe(Book b,SortedSet<Integer>ids){super(b,ids);}
 public void process(){try{
  Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");call(cls("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);
  for(Object sys:(List<?>)call(sheet,"getSystems"))call(construct("org.audiveris.omr.sheet.header.HeaderBuilder",new Class<?>[]{cls("org.audiveris.omr.sheet.SystemInfo")},sys),"processHeader");
  call(cls("org.audiveris.omr.sheet.stem.StemSeedsStep").getConstructor().newInstance(),"doit",sheet);
  Object picture=call(sheet,"getPicture"),image=call(picture,"getSource",enumValue("org.audiveris.omr.sheet.Picture$SourceKey","NO_STAFF"));Object builder=construct("org.audiveris.omr.sheet.beam.SpotsBuilder",new Class<?>[]{cls("org.audiveris.omr.sheet.Sheet")},sheet);Object buffer=call(builder,"getBuffer");String before=mask(buffer);call(builder,"eraseHeaderAreas",buffer);String erased=mask(buffer);Object scale=call(sheet,"getScale");int beam=(Integer)call(scale,"getItemValue",enumValue("org.audiveris.omr.sheet.Scale$Item","beam"));List<?> extracted=(List<?>)call(builder,"buildSpots",buffer,null,(double)beam,null);call(builder,"dispatchSheetSpots",extracted);List<String> systems=new ArrayList<>();for(Object sys:(List<?>)call(sheet,"getSystems"))systems.add("{\"system_id\":"+call(sys,"getId")+",\"grouped_spots\":"+spots((Collection<?>)call(sys,"getGroupedGlyphs",enumValue("org.audiveris.omr.glyph.GlyphGroup","BEAM_SPOT")))+"}");
  Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"native_outputs_used\":false,\"scope\":\"Actual original GRID/live HEADERS/STEM_SEEDS then SpotsBuilder.buildSpots and dispatchSheetSpots; original grouped spot pixels/order; no beam interpretation or rest claim\",\"header_erase_rectangles\":"+headerEraseRectangles(sheet)+",\"no_staff\":"+mask(image)+",\"gaussian\":"+before+",\"header_erased\":"+erased+",\"sheet_spots\":"+spots(extracted)+",\"systems\":["+String.join(",",systems)+"]}\n");
 }catch(Exception e){throw new RuntimeException(e);}}
}
