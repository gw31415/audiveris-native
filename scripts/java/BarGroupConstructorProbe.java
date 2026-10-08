// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.awt.geom.Line2D;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class BarGroupConstructorProbe extends VoiceLifecycleProbe {
 public static void main(String[]args)throws Exception{
  Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0])),sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),staff=((List<?>)call(sys,"getStaves")).get(1);List<Object> cases=new ArrayList<>();
  for(List<String> shapes:List.of(List.of("THIN_BARLINE"),List.of("THICK_BARLINE"),List.of("THIN_BARLINE","THIN_BARLINE"),List.of("THIN_BARLINE","THICK_BARLINE"),List.of("THICK_BARLINE","THIN_BARLINE"),List.of("THICK_BARLINE","THICK_BARLINE"),List.of("THIN_BARLINE","THICK_BARLINE","THIN_BARLINE"),List.of("THIN_BARLINE","THICK_BARLINE","THICK_BARLINE","THIN_BARLINE"))){
   Object sig=make("sig.SIGraph",sys);List<Object> bars=new ArrayList<>(),inputs=new ArrayList<>();for(int i=0;i<shapes.size();i++){double width=shapes.get(i).equals("THIN_BARLINE")?2.:4.;Object n=make("sig.inter.BarlineInter",null,c("glyph.Shape").getField(shapes.get(i)).get(null),.8-i*.1,new Line2D.Double(300+i*7,500,300+i*7,545),width);call(n,"setStaff",staff);call(sig,"addVertex",n);if(i==1)call(n,"freeze");if(i==shapes.size()-1)call(n,"setManual",true);bars.add(n);inputs.add(obj("bounds",rect(call(n,"getBounds")),"shape",shapes.get(i),"grade",call(n,"getGrade"),"frozen",call(n,"isFrozen"),"manual",call(n,"isManual")));}
   Object sb=make("sig.inter.StaffBarlineInter",bars);List<?> members=(List<?>)call(sb,"getMembers");boolean same=true;for(int i=0;i<bars.size();i++)same &= members.get(i)==bars.get(i);cases.add(obj("members",inputs,"bounds",rect(call(sb,"getBounds")),"style",call(sb,"getStyle"),"grade",call(sb,"getGrade"),"frozen",call(sb,"isFrozen"),"manual",call(sb,"isManual"),"back_to_back",call(sb,"isBackToBack"),"left_x",call(sb,"getLeftX"),"right_x",call(sb,"getRightX"),"same_member_objects",same,"same_staff",call(sb,"getStaff")==staff));
  }
  new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","declared physical Barline objects on original actual Staff and isolated SIG; untouched original StaffBarline constructor, member references, grade/frozen/manual/style and left/right endpoints including original null four-bar style","cases",cases));System.exit(0);
 }
}
