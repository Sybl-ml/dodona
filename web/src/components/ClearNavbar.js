import React from "react"
import { Navbar} from 'react-bootstrap';
import styled from "styled-components";
import MemoLogo from '../icons/Logo.js';

const ClearHeaderBar = styled(Navbar)`
	min-height: 4rem;
    background: linear-gradient(${({ theme }) => theme.body}, ${({ theme }) => theme.body} 90%, transparent 100%);
    transition: all 0.25s linear;
`;


const ClearHeader = ({theme}) => {
        return (
		<ClearHeaderBar sticky="top">
			<ClearHeaderBar.Brand href="/home">
				<MemoLogo 
					theme={theme} 
				/> 
			</ClearHeaderBar.Brand>
				
  		</ClearHeaderBar>
        );
    };
export default ClearHeader;