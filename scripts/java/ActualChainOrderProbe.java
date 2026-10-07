// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;import java.util.*;import com.fasterxml.jackson.databind.ObjectMapper;
public class ActualChainOrderProbe extends DotCallbackOrderProbe {
 public static void main(String[] args)throws Exception {
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),dd=null,aug=null;
  for(Object e:(Collection<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("DoubleDotRelation")){dd=e;break;}
  Object first=call(sig,"getEdgeTarget",dd),second=call(sig,"getEdgeSource",dd);
  for(Object e:(Collection<?>)call(sig,"edgesOf",first))if(e.getClass().getSimpleName().equals("AugmentationRelation")){aug=e;break;}
  Object head=call(sig,"getEdgeTarget",aug),chord=call(head,"getChord");List<Object> notes=List.of(head),owners=List.of(chord),dots=List.of(first,second),edges=List.of(aug,dd),trace=new ArrayList<>();
  call(chord,"countDots");Object before=state(sig,notes,owners,dots,edges);watch(sig,notes,owners,dots,edges,trace);call(first,"remove");Object after=state(sig,notes,owners,dots,edges);
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("original_source","9e1e55cd2746037d059345881c53e6a6754bffbd","scope","actual saved successful double-dot first vertex removal, in-place SigListener wrapper preserving event order; not export after mutation","notes",List.of(obj("kind","HeadInter","bounds",rect(call(head,"getBounds")),"shape",call(head,"getShape").toString())),"owner_bounds",List.of(rect(call(chord,"getBounds"))),"dot_bounds",List.of(rect(call(first,"getBounds")),rect(call(second,"getBounds"))),"before",before,"trace",trace,"after",after));System.exit(0);
 }
}
