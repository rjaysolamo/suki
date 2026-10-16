import type { Metadata } from 'next';
import './globals.css';
export const metadata: Metadata = {title:'Suki — Your market, delivered.',description:'Fresh finds from Bankerohan Public Market, delivered to your doorstep in Davao City.'};
export default function RootLayout({children}:{children:React.ReactNode}) {return <html lang="en"><body>{children}</body></html>}
