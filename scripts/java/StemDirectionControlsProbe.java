// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class StemDirectionControlsProbe extends FoundationOverlapProbe {
 public static void main(String[] args)throws Exception {
  c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
  Map<String,Object> rows=new LinkedHashMap<>();String[] names={"up","down","equal-distance-bottom-up-tie","stemless"};int[][] ys={{120,140},{80,100},{86,126},{120}};
  for(int i=0;i<names.length;i++) {
   Object chord=make("sig.inter.HeadChordInter",.8);call(chord,"setStaff",call(sys,"getFirstStaff"));call(sig,"addVertex",chord);
   if(i<3){Object stem=make("sig.inter.StemInter",physical(100,80,2,60,"VERTICAL",null),.8);call(sig,"addVertex",stem);call(chord,"setStem",stem);}
   for(int y:ys[i]){Object h=head(sys,physical(88,y,12,8,"VERTICAL",null));call(sig,"addVertex",h);call(chord,"addMember",h);}
   rows.put(names[i],call(chord,"getStemDir"));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","four declared original typed chord stem direction cases; independent from successful image comparison","directions",rows));System.exit(0);
 }
}
