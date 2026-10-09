// SPDX-License-Identifier: AGPL-3.0-or-later
import org.audiveris.proxymusic.ScorePartwise;
import org.audiveris.proxymusic.Note;
import org.audiveris.proxymusic.util.Marshalling;
import jakarta.xml.bind.JAXBElement;
import java.nio.file.*;
import java.util.*;
import javax.xml.stream.*;
public final class ReadMusicXml {
 public static void main(String[] args) throws Exception {
  XMLInputFactory f=XMLInputFactory.newFactory();f.setProperty(XMLInputFactory.SUPPORT_DTD,false);f.setProperty(XMLInputFactory.IS_SUPPORTING_EXTERNAL_ENTITIES,false);
  Object root;try(var in=Files.newInputStream(Path.of(args[0]))){var reader=f.createXMLStreamReader(in);root=Marshalling.getContext(ScorePartwise.class).createUnmarshaller().unmarshal(reader);reader.close();}
  if(root instanceof JAXBElement<?> e)root=e.getValue();if(!(root instanceof ScorePartwise score))throw new IllegalStateException("not partwise");
  int notes=0,rests=0,chords=0,beams=0,measures=0,graces=0;Set<String> voices=new TreeSet<>();List<String> partRows=new ArrayList<>();
  for(var part:score.getPart()){int partNotes=0;for(var m:part.getMeasure()){measures++;for(var x:m.getNoteOrBackupOrForward()){if(x instanceof Note n){notes++;partNotes++;if(n.getRest()!=null)rests++;if(n.getChord()!=null)chords++;beams+=n.getBeam().size();if(n.getGrace()!=null)graces++;else if(n.getDuration()==null||n.getDuration().signum()<=0)throw new IllegalStateException("invalid duration");if(n.getRest()==null&&n.getPitch()==null&&n.getUnpitched()==null)throw new IllegalStateException("missing pitch");if(n.getVoice()!=null)voices.add(n.getVoice());}}}partRows.add("{\"measures\":"+part.getMeasure().size()+",\"notes\":"+partNotes+"}");}
  List<String> v=new ArrayList<>();for(String voice:voices)v.add("\""+voice+"\"");System.out.println("{\"reader\":\"ProxyMusic4.0.3 typed JAXB/StAX, DTD and external entities disabled\",\"parts\":["+String.join(",",partRows)+"],\"measures\":"+measures+",\"notes\":"+notes+",\"rests\":"+rests+",\"chord_continuations\":"+chords+",\"beams\":"+beams+",\"graces\":"+graces+",\"voices\":["+String.join(",",v)+"]}");
 }
}
