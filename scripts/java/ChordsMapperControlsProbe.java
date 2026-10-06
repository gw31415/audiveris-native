// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class ChordsMapperControlsProbe extends RookieMappingProbe {
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),measure=call(((List<?>)call(sys,"getStacks")).get(1),"getFirstMeasure");call(measure,"resetRhythm");
  List<?> chords=new ArrayList<>((Collection<?>)call(measure,"getStandardChords"));Object a=chords.get(0),r=chords.get(1),vd=make("sheet.rhythm.VoiceDistance$Separated",call(sheet,"getScale")),pair=make("sheet.rhythm.ChordPair",r,a,(Object)null);List<Object> cases=new ArrayList<>();
  for(boolean next:new boolean[]{false,true}){Set<Object> black=new LinkedHashSet<>(List.of(pair)),nexts=next?new LinkedHashSet<>(List.of(pair)):new LinkedHashSet<>();Object result=call(make("sheet.rhythm.ChordsMapper",List.of(r),List.of(a),vd,black,nexts),"process");List<Object> pairs=new ArrayList<>();for(Object p:(Collection<?>)field(result,"pairs"))pairs.add(obj("rookie",chords.indexOf(field(p,"rookie")),"active",chords.indexOf(field(p,"active"))));cases.add(obj("name",next?"explicit-next-preassignment-before-black":"black-forces-no-link","rookies",List.of(1),"actives",List.of(0),"distance",call(vd,"getDistance",a,r,(Object)null),"black",List.of(obj("rookie",1,"active",0)),"next",next?List.of(obj("rookie",1,"active",0)):List.of(),"mapping",obj("cost",field(result,"cost"),"pairs",pairs)));}
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","two declared original ChordsMapper constraint-list controls on actual saved chord objects; not image automatic constraint acquisition","cases",cases));System.exit(0);
 }
}
