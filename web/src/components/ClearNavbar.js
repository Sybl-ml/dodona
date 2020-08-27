import React from "react"
import { Navbar, Nav, NavDropdown,} from 'react-bootstrap';
import styled from "styled-components";
import MemoLogo from '../icons/Logo.js';
import {PrimaryButton} from './Buttons';

const ClearHeaderBar = styled(Navbar)`
	min-height: 4rem;
    background: linear-gradient(${({ theme }) => theme.body}, ${({ theme }) => theme.body} 90%, transparent 100%);
    transition: all 0.25s linear;
`;


const Square = styled.div`
	position: absolute;
	top: -5%;
	left: 45%;
	width: 10%;
	height: 10%;
	filter: drop-shadow(0 1px 2px 0 0 rgba(0,0,0,.5));
	background-color:${({ theme }) => theme.body};
	transform: rotate(45deg);
`;


const ClearHeader = ({theme}) => {
        return (
		<ClearHeaderBar sticky="top">
			<ClearHeaderBar.Brand href="/home">
				<MemoLogo 
					theme={theme} 
				/> 
			</ClearHeaderBar.Brand>
				
    		<ClearHeaderBar.Collapse className="justify-content-end">
				<PrimaryButton variant="primary" href="/register">SIGN UP NOW</PrimaryButton>
			</ClearHeaderBar.Collapse>
  		</ClearHeaderBar>
        );
    };
export default ClearHeader;